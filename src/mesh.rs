use crate::bounding_box::Box;
use crate::cube::Cube;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::tree::Tree;
use crate::triangle::Triangle;
use crate::vector::Vector;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Triangle mesh shape.
pub struct Mesh {
    bx: Box,
    vertices: Vec<Vector>,
    index_triangles: Vec<IndexTriangle>,
    tree: Option<Tree>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let mut merger = VertexMerger::new(1e-6);
        let itriangles = triangles
            .iter()
            .map(|t| IndexTriangle {
                v1: merger.get_or_insert(t.v1),
                v2: merger.get_or_insert(t.v2),
                v3: merger.get_or_insert(t.v3),
            })
            .collect();
        Mesh {
            bx: Box::for_shapes(triangles.into_iter()),
            index_triangles: itriangles,
            vertices: merger.vertices,
            tree: None,
        }
    }

    pub fn triangles(&self) -> impl Iterator<Item = Triangle> + ExactSizeIterator {
        self.index_triangles.iter().map(|itr| {
            Triangle::new(
                self.vertices[itr.v1],
                self.vertices[itr.v2],
                self.vertices[itr.v3],
            )
        })
    }

    pub fn unit_cube(self) -> Self {
        self.fit_inside(
            Box::new(Vector::default(), Vector::new(1.0, 1.0, 1.0)),
            Vector::default(),
        )
        .move_to(Vector::default(), Vector::new(0.5, 0.5, 0.5))
    }

    pub fn move_to(self, position: Vector, anchor: Vector) -> Self {
        let matrix = Matrix::translate(position.sub(self.bx.anchor(anchor)));
        self.transform(&matrix)
    }

    pub fn fit_inside(self, bx: Box, anchor: Vector) -> Self {
        let scale = bx.size().div(self.bx.size()).min_component();
        let extra = bx.size().sub(self.bx.size().mul_scalar(scale));
        let mut matrix = Matrix::identity();
        matrix = matrix.translated(self.bx.min.mul_scalar(-1.0));
        matrix = matrix.scaled(Vector::new(scale, scale, scale));
        matrix = matrix.translated(bx.min.add(extra.mul(anchor)));
        self.transform(&matrix)
    }

    pub fn transform(mut self, matrix: &Matrix) -> Self {
        for v in self.vertices.iter_mut() {
            *v = matrix.mul_position(*v);
        }
        self.bx = Box::for_shapes(self.triangles());
        self.tree = None;
        self
    }

    pub fn voxelize(&self, size: f64) -> Vec<Cube> {
        let z1 = self.bx.min.z;
        let z2 = self.bx.max.z;
        let mut set: HashSet<(i64, i64, i64)> = HashSet::new();

        let mut z = z1;
        while z <= z2 {
            let plane = Plane::new(Vector::new(0.0, 0.0, z), Vector::new(0.0, 0.0, 1.0));
            let paths = plane.intersect_mesh(self);
            for path in &paths.paths {
                for v in path {
                    let x = ((v.x / size + 0.5).floor() * size * 1000.0) as i64;
                    let y = ((v.y / size + 0.5).floor() * size * 1000.0) as i64;
                    let z = ((v.z / size + 0.5).floor() * size * 1000.0) as i64;
                    set.insert((x, y, z));
                }
            }
            z += size;
        }

        set.into_iter()
            .map(|(x, y, z)| {
                let v = Vector::new(x as f64 / 1000.0, y as f64 / 1000.0, z as f64 / 1000.0);
                Cube::new(v.sub_scalar(size / 2.0), v.add_scalar(size / 2.0))
            })
            .collect()
    }
}

impl Shape for Mesh {
    fn compile(&mut self) {
        if self.tree.is_none() {
            self.tree = Some(Tree::new(
                self.triangles()
                    .into_iter()
                    .map(|t| Arc::new(t) as Arc<dyn Shape + Send + Sync>)
                    .collect(),
            ));
        }
    }

    fn bounding_box(&self) -> Box {
        self.bx
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.tree
            .as_ref()
            .map_or(Hit::no_hit(), |tree| tree.intersect(r))
    }

    fn paths(&self) -> Paths {
        let mut normal_merger = VertexMerger::new(1e-6);
        let mut counter = HashMap::new();
        self.index_triangles.iter().for_each(|it| {
            let normal = normal_merger.get_or_insert(it.normalized_normal(&self.vertices));
            it.paths().into_iter().for_each(|path| {
                counter
                    .entry((path, normal))
                    .and_modify(|i| *i += 1)
                    .or_insert(1);
            })
        });
        Paths::from_vec(
            counter
                .into_iter()
                .filter(|(_, count)| *count == 1)
                .map(|(((a, b), _), _)| vec![self.vertices[a], self.vertices[b]])
                .collect(),
        )
    }
}

/// Triangle defined by indices into a vertex list.
struct IndexTriangle {
    v1: usize,
    v2: usize,
    v3: usize,
}

impl IndexTriangle {
    fn paths(&self) -> [(usize, usize); 3] {
        let [v1, v2, v3] = {
            let mut vs = [self.v1, self.v2, self.v3];
            vs.sort();
            vs
        };
        [(v1, v2), (v2, v3), (v1, v3)]
    }

    fn normalized_normal(&self, vertices: &[Vector]) -> Vector {
        let v1 = vertices[self.v1];
        let v2 = vertices[self.v2];
        let v3 = vertices[self.v3];
        let normal = (v2.sub(v1)).cross(v3.sub(v1)).normalize();
        if normal.x < 0.0
            || (normal.x == 0.0 && normal.y < 0.0)
            || (normal.x == 0.0 && normal.y == 0.0 && normal.z < 0.0)
        {
            normal.mul_scalar(-1.0)
        } else {
            normal
        }
    }
}

struct VertexMerger {
    pub vertices: Vec<Vector>,
    grid: HashMap<(i64, i64, i64), Vec<usize>>,
    epsilon: f64,
    epsilon_sq: f64,
}

impl VertexMerger {
    pub fn new(epsilon: f64) -> Self {
        Self {
            vertices: Vec::new(),
            grid: HashMap::new(),
            epsilon,
            epsilon_sq: epsilon * epsilon,
        }
    }

    /// Returns the index of the existing vertex if it's close enough,
    /// or inserts a new vertex and returns its index.
    pub fn get_or_insert(&mut self, v: Vector) -> usize {
        let cell_size = self.epsilon;

        let ix = (v.x / cell_size).floor() as i64;
        let iy = (v.y / cell_size).floor() as i64;
        let iz = (v.z / cell_size).floor() as i64;

        let dxyz = (-1..=1)
            .flat_map(|dx| (-1..=1).flat_map(move |dy| (-1..=1).map(move |dz| (dx, dy, dz))));
        for (dx, dy, dz) in dxyz {
            let key = (ix + dx, iy + dy, iz + dz);

            if let Some(indices) = self.grid.get(&key) {
                for &idx in indices {
                    let existing_v = self.vertices[idx];

                    if v.distance_squared(existing_v) < self.epsilon_sq {
                        return idx;
                    }
                }
            }
        }

        let new_idx = self.vertices.len();
        self.vertices.push(v);

        self.grid
            .entry((ix, iy, iz))
            .or_insert_with(Vec::new)
            .push(new_idx);

        new_idx
    }
}

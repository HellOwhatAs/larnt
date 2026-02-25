use crate::bounding_box::BBox;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::tree::Tree;
use crate::triangle::Triangle;
use crate::vector::Vector;
use std::collections::HashMap;

/// Triangle mesh shape.
pub struct Mesh {
    bx: BBox,
    vertices: Vec<Vector>,
    triangles: Vec<usize>,
    tree: Tree<Triangle>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vector>, triangles: Vec<usize>) -> Self {
        let tree = Tree::new(
            triangles
                .chunks_exact(3)
                .map(|w| Triangle::new(vertices[w[0]], vertices[w[1]], vertices[w[2]]))
                .collect(),
        );
        Self {
            bx: BBox::for_vectors(&vertices),
            triangles,
            vertices,
            tree,
        }
    }

    pub fn from_triangles(triangles: Vec<Triangle>) -> Self {
        let mut merger = VertexMerger::new(1e-6);
        let itriangles = triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3].map(|v| merger.get_or_insert(v)))
            .collect();
        Self {
            bx: BBox::for_shapes(&triangles),
            triangles: itriangles,
            vertices: merger.vertices,
            tree: Tree::new(triangles),
        }
    }

    pub fn triangles(&self) -> &[Triangle] {
        self.tree.shapes()
    }

    pub fn unit_cube(self) -> Self {
        self.fit_inside(
            BBox::new(Vector::default(), Vector::new(1.0, 1.0, 1.0)),
            Vector::default(),
        )
        .move_to(Vector::default(), Vector::new(0.5, 0.5, 0.5))
    }

    pub fn move_to(self, position: Vector, anchor: Vector) -> Self {
        let matrix = Matrix::translate(position.sub(self.bx.anchor(anchor)));
        self.transform(&matrix)
    }

    pub fn fit_inside(self, bx: BBox, anchor: Vector) -> Self {
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
        self.tree = Tree::new(
            self.triangles
                .chunks_exact(3)
                .map(|w| {
                    Triangle::new(
                        self.vertices[w[0]],
                        self.vertices[w[1]],
                        self.vertices[w[2]],
                    )
                })
                .collect(),
        );
        self.bx = BBox::for_shapes(self.triangles());
        self
    }

    pub fn parametric_surface(
        points: Vec<Vector>,
        u_steps: usize,
        v_steps: usize,
        indexer: impl Fn(usize, usize) -> usize,
    ) -> Mesh {
        let mut triangles = Vec::with_capacity(u_steps * v_steps * 6);
        for u in 0..u_steps {
            for v in 0..v_steps {
                let i00 = indexer(u, v);
                let i10 = indexer(u + 1, v);
                let i01 = indexer(u, v + 1);
                let i11 = indexer(u + 1, v + 1);

                for i123 in [[i00, i10, i01], [i10, i11, i01]] {
                    let [p1, p2, p3] = i123.map(|i| points[i]);
                    let cross = (p2.sub(p1)).cross(p3.sub(p1));
                    let area_squared = cross.x * cross.x + cross.y * cross.y + cross.z * cross.z;
                    if area_squared > crate::common::EPS {
                        triangles.extend(i123);
                    }
                }
            }
        }
        Mesh::new(points, triangles)
    }
}

impl Shape for Mesh {
    fn bounding_box(&self) -> BBox {
        self.bx
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.tree.intersect(r)
    }

    fn paths(&self, _args: &RenderArgs) -> Paths {
        let mut paths = Paths::new();
        let mut normal_merger = VertexMerger::new(1e-6);
        let mut counter = HashMap::new();
        self.triangles.chunks_exact(3).for_each(|it| {
            let normal = normal_merger
                .get_or_insert(normalized_normal(it.iter().map(|&i| self.vertices[i])));
            triangle_paths(it).into_iter().for_each(|path| {
                counter
                    .entry((path, normal))
                    .and_modify(|i| *i += 1)
                    .or_insert(1);
            })
        });

        counter
            .into_iter()
            .filter(|(_, count)| *count == 1)
            .for_each(|(((a, b), _), _)| {
                paths
                    .new_path()
                    .extend([self.vertices[a], self.vertices[b]])
            });

        paths
    }
}

#[inline(always)]
fn triangle_paths(i123: &[usize]) -> [(usize, usize); 3] {
    let mut i123 = [0, 1, 2].map(|i| i123[i]);
    i123.sort_unstable();
    let [i1, i2, i3] = i123;
    [(i1, i2), (i2, i3), (i1, i3)]
}

fn normalized_normal(mut v123: impl Iterator<Item = Vector>) -> Vector {
    let [v1, v2, v3] = std::array::from_fn(|_| v123.next().unwrap());
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

        self.grid.entry((ix, iy, iz)).or_default().push(new_idx);

        new_idx
    }
}

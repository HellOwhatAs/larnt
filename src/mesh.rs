use crate::bounding_box::BBox;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::tree::Tree;
use crate::triangle::Triangle;
use crate::vector::Vector;
use crate::{Matrix, TransformedShape};
use bon::Builder;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub enum MeshTexture {
    #[default]
    Triangles,
    Polygonal,
    Silhouette,
}

/// Triangle mesh shape.
#[derive(Builder)]
pub struct Mesh {
    #[builder(start_fn)]
    pub vertices: Vec<Vector>,
    #[builder(start_fn)]
    pub triangles: Vec<usize>,
    #[builder(skip = BBox::for_vectors(&vertices))]
    pub bx: BBox,
    #[builder(default)]
    pub flipped_triangles: HashSet<(usize, usize)>,
    #[builder(default)]
    pub texture: MeshTexture,
    #[builder(skip = Tree::new(
        triangles
            .chunks_exact(3)
            .map(|w| Triangle::new(vertices[w[0]], vertices[w[1]], vertices[w[2]]))
            .collect(),
    ))]
    tree: Tree<Triangle>,
}

impl Mesh {
    pub fn from_triangles(triangles: Vec<Triangle>) -> Self {
        let mut merger = VertexMerger::new(1e-6);
        let itriangles = triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3].map(|v| merger.get_or_insert(v)))
            .collect();
        Self::builder(merger.vertices, itriangles).build()
    }

    pub fn fit_inside(&self, bx: BBox, anchor: Vector) -> Matrix {
        let scale = bx.size().div(self.bx.size()).min_component();
        let extra = bx.size().sub(self.bx.size().mul_scalar(scale));
        let mut matrix = Matrix::identity();
        matrix = matrix.translated(self.bx.min.mul_scalar(-1.0));
        matrix = matrix.scaled(Vector::new(scale, scale, scale));
        matrix = matrix.translated(bx.min.add(extra.mul(anchor)));
        matrix
    }

    pub fn filter_paths(
        &self,
        group_keeper: impl Fn(&[(usize, usize, usize)]) -> bool,
    ) -> Paths<usize> {
        let mut paths = Paths::new();
        if self.triangles.len() < 3 {
            return paths;
        }

        let edges = {
            let mut edges = self
                .triangles
                .chunks_exact(3)
                .enumerate()
                .flat_map(|(face_idx, chunk)| {
                    let [i1, i2, i3] = [chunk[0], chunk[1], chunk[2]];
                    [
                        (i1.min(i2), i1.max(i2), face_idx),
                        (i2.min(i3), i2.max(i3), face_idx),
                        (i3.min(i1), i3.max(i1), face_idx),
                    ]
                })
                .collect::<Vec<_>>();
            edges.sort_unstable_by_key(|e| (e.0, e.1));
            edges
        };

        let mut i = 0;
        while i < edges.len() {
            let current_edge = (edges[i].0, edges[i].1);
            let mut count = 1;

            while i + count < edges.len()
                && (edges[i + count].0, edges[i + count].1) == current_edge
            {
                count += 1;
            }

            if group_keeper(&edges[i..i + count]) {
                paths.new_path().extend([current_edge.0, current_edge.1]);
            }

            i += count;
        }

        paths
    }

    pub fn triangle_paths(&self, _args: &RenderArgs) -> Paths<Vector> {
        self.filter_paths(|_| true)
            .splice_exact()
            .map(|i| self.vertices[i])
    }

    pub fn polygonal_paths(&self, _args: &RenderArgs) -> Paths<Vector> {
        let face_normals: Vec<Vector> = self
            .triangles
            .chunks_exact(3)
            .map(|chunk| normalized_normal(chunk.iter().map(|&i| self.vertices[i])))
            .collect();
        self.filter_paths(|edges| {
            if edges.len() == 1 {
                true
            } else {
                let base_normal = face_normals[edges[0].2];
                edges.iter().skip(1).any(|e| {
                    let other_normal = face_normals[e.2];
                    base_normal.distance_squared(other_normal) > crate::common::EPS
                })
            }
        })
        .splice_exact()
        .map(|i| self.vertices[i])
    }

    pub fn silhouette_paths(&self, args: &RenderArgs) -> Paths<Vector> {
        let face_data: Vec<_> = self
            .triangles
            .chunks_exact(3)
            .map(|chunk| {
                let [v1, v2, v3] = [0, 1, 2].map(|i| self.vertices[chunk[i]]);
                let true_normal = (v2.sub(v1)).cross(v3.sub(v1)).normalize();
                let view_dir = args.eye.sub(v1);
                true_normal.dot(view_dir)
            })
            .collect();
        self.filter_paths(|edges| {
            if edges.len() == 2 {
                let [dot1, dot2] = [0, 1].map(|i| face_data[edges[i].2]);
                let (a, b) = (edges[0].2, edges[1].2);
                let key = if a < b { (a, b) } else { (b, a) };
                if self.flipped_triangles.contains(&key) {
                    return dot1 * dot2 >= 0.0;
                }
                dot1 * dot2 <= 0.0
            } else {
                true
            }
        })
        .splice_exact()
        .map(|i| self.vertices[i])
    }
}

impl AsRef<Triangle> for Triangle {
    fn as_ref(&self) -> &Triangle {
        self
    }
}

pub trait TriangleMesh {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator;
}

impl TriangleMesh for Mesh {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator {
        self.tree.shapes().iter()
    }
}

impl TriangleMesh for TransformedShape<Mesh> {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator {
        self.shape.tree.shapes().iter().map(|triangle| {
            Triangle::new(
                self.matrix.mul_position(triangle.v1),
                self.matrix.mul_position(triangle.v2),
                self.matrix.mul_position(triangle.v3),
            )
        })
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

    fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
        match self.texture {
            MeshTexture::Triangles => self.triangle_paths(args),
            MeshTexture::Polygonal => self.polygonal_paths(args),
            MeshTexture::Silhouette => self.silhouette_paths(args),
        }
    }
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

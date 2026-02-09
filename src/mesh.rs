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
use std::collections::HashSet;
use std::sync::Arc;

struct IndexTriangle {
    v1: usize,
    v2: usize,
    v3: usize,
}

impl IndexTriangle {
    fn to_triangle(&self, vertices: &[Vector]) -> Triangle {
        Triangle::new(vertices[self.v1], vertices[self.v2], vertices[self.v3])
    }

    fn paths(&self) -> [(usize, usize); 3] {
        let [v1, v2, v3] = {
            let mut vs = [self.v1, self.v2, self.v3];
            vs.sort();
            vs
        };
        [(v1, v2), (v2, v3), (v1, v3)]
    }
}

pub struct Mesh {
    bx: Box,
    vertices: Vec<Vector>,
    itriangles: Vec<IndexTriangle>,
    tree: Option<Arc<Tree>>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let bx = Box::for_triangles(&triangles);
        let vertices: Vec<Vector> = triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3])
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let itriangles = triangles
            .into_iter()
            .map(|t| {
                let v1 = vertices.iter().position(|&v| v == t.v1).unwrap();
                let v2 = vertices.iter().position(|&v| v == t.v2).unwrap();
                let v3 = vertices.iter().position(|&v| v == t.v3).unwrap();
                IndexTriangle { v1, v2, v3 }
            })
            .collect();
        Mesh {
            bx,
            itriangles,
            vertices,
            tree: None,
        }
    }

    pub fn num_triangles(&self) -> usize {
        self.itriangles.len()
    }

    pub fn triangles(&self) -> Vec<Triangle> {
        self.itriangles
            .iter()
            .map(|it| it.to_triangle(&self.vertices))
            .collect()
    }
}

impl Shape for Mesh {
    fn compile(&mut self) {
        if self.tree.is_none() {
            let shapes: Vec<Arc<dyn Shape + Send + Sync>> = self
                .triangles()
                .iter()
                .map(|t| Arc::new(t.clone()) as Arc<dyn Shape + Send + Sync>)
                .collect();
            self.tree = Some(Arc::new(Tree::new(shapes)));
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
        Paths::from_vec(
            self.itriangles
                .iter()
                .flat_map(|it| it.paths())
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|(a, b)| vec![self.vertices[a], self.vertices[b]])
                .collect(),
        )
    }
}

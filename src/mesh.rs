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

pub struct Mesh {
    pub bx: Box,
    pub triangles: Vec<Triangle>,
    tree: Option<Arc<Tree>>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let bx = Box::for_triangles(&triangles);
        Mesh {
            bx,
            triangles,
            tree: None,
        }
    }

    pub fn update_bounding_box(&mut self) {
        self.bx = Box::for_triangles(&self.triangles);
    }

    pub fn unit_cube(&mut self) {
        self.fit_inside(Box::new(Vector::default(), Vector::new(1.0, 1.0, 1.0)), Vector::default());
        self.move_to(Vector::default(), Vector::new(0.5, 0.5, 0.5));
    }

    pub fn move_to(&mut self, position: Vector, anchor: Vector) {
        let matrix = Matrix::translate(position.sub(self.bx.anchor(anchor)));
        self.transform(&matrix);
    }

    pub fn fit_inside(&mut self, bx: Box, anchor: Vector) {
        let scale = bx.size().div(self.bx.size()).min_component();
        let extra = bx.size().sub(self.bx.size().mul_scalar(scale));
        let mut matrix = Matrix::identity();
        matrix = matrix.translated(self.bx.min.mul_scalar(-1.0));
        matrix = matrix.scaled(Vector::new(scale, scale, scale));
        matrix = matrix.translated(bx.min.add(extra.mul(anchor)));
        self.transform(&matrix);
    }

    pub fn transform(&mut self, matrix: &Matrix) {
        for t in &mut self.triangles {
            t.v1 = matrix.mul_position(t.v1);
            t.v2 = matrix.mul_position(t.v2);
            t.v3 = matrix.mul_position(t.v3);
            t.update_bounding_box();
        }
        self.update_bounding_box();
        self.tree = None;
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
            let shapes: Vec<Arc<dyn Shape + Send + Sync>> = self.triangles.iter()
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
        self.tree.as_ref().map_or(Hit::no_hit(), |tree| tree.intersect(r))
    }

    fn paths(&self) -> Paths {
        let mut result = Paths::new();
        for t in &self.triangles {
            result.extend(t.paths());
        }
        result
    }
}

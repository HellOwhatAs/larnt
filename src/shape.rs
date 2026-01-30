use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::vector::Vector;

pub trait Shape {
    fn compile(&mut self) {}
    fn bounding_box(&self) -> Box;
    fn contains(&self, v: Vector, f: f64) -> bool;
    fn intersect(&self, r: Ray) -> Hit;
    fn paths(&self) -> Paths;
}

#[derive(Debug, Clone, Default)]
pub struct EmptyShape;

impl Shape for EmptyShape {
    fn bounding_box(&self) -> Box {
        Box::new(Vector::default(), Vector::default())
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, _r: Ray) -> Hit {
        Hit::no_hit()
    }

    fn paths(&self) -> Paths {
        Paths::new()
    }
}

pub struct TransformedShape {
    pub shape: std::sync::Arc<dyn Shape + Send + Sync>,
    pub matrix: Matrix,
    pub inverse: Matrix,
}

impl TransformedShape {
    pub fn new(shape: std::sync::Arc<dyn Shape + Send + Sync>, matrix: Matrix) -> Self {
        let inverse = matrix.inverse();
        TransformedShape { shape, matrix, inverse }
    }
}

impl Shape for TransformedShape {
    fn bounding_box(&self) -> Box {
        self.matrix.mul_box(self.shape.bounding_box())
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.shape.contains(self.inverse.mul_position(v), f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.shape.intersect(self.inverse.mul_ray(r))
    }

    fn paths(&self) -> Paths {
        self.shape.paths().transform(&self.matrix)
    }
}

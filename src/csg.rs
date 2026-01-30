use crate::bounding_box::Box;
use crate::filter::Filter;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{EmptyShape, Shape};
use crate::vector::Vector;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Intersection,
    Difference,
}

pub struct BooleanShape {
    pub op: Op,
    pub a: Arc<dyn Shape + Send + Sync>,
    pub b: Arc<dyn Shape + Send + Sync>,
}

impl BooleanShape {
    pub fn new(op: Op, a: Arc<dyn Shape + Send + Sync>, b: Arc<dyn Shape + Send + Sync>) -> Self {
        BooleanShape { op, a, b }
    }
}

pub fn new_boolean_shape(op: Op, shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Arc<dyn Shape + Send + Sync> {
    if shapes.is_empty() {
        return Arc::new(EmptyShape);
    }
    let mut shape: Arc<dyn Shape + Send + Sync> = shapes[0].clone();
    for s in shapes.into_iter().skip(1) {
        shape = Arc::new(BooleanShape::new(op, shape, s));
    }
    shape
}

pub fn new_intersection(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Arc<dyn Shape + Send + Sync> {
    new_boolean_shape(Op::Intersection, shapes)
}

pub fn new_difference(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Arc<dyn Shape + Send + Sync> {
    new_boolean_shape(Op::Difference, shapes)
}

impl Shape for BooleanShape {
    fn bounding_box(&self) -> Box {
        let a = self.a.bounding_box();
        let b = self.b.bounding_box();
        a.extend(b)
    }

    fn contains(&self, v: Vector, _f: f64) -> bool {
        let f = 1e-3;
        match self.op {
            Op::Intersection => self.a.contains(v, f) && self.b.contains(v, f),
            Op::Difference => self.a.contains(v, f) && !self.b.contains(v, -f),
        }
    }

    fn intersect(&self, r: Ray) -> Hit {
        let h1 = self.a.intersect(r);
        let h2 = self.b.intersect(r);
        let h = h1.min(h2);
        let v = r.position(h.t);
        
        if !h.is_ok() || self.contains(v, 0.0) {
            return h;
        }
        
        self.intersect(Ray::new(r.position(h.t + 0.01), r.direction))
    }

    fn paths(&self) -> Paths {
        let mut p = self.a.paths();
        p.extend(self.b.paths());
        p = p.chop(0.01);
        p = p.filter(self);
        p
    }
}

impl Filter for BooleanShape {
    fn filter(&self, v: Vector) -> Option<Vector> {
        if self.contains(v, 0.0) {
            Some(v)
        } else {
            None
        }
    }
}

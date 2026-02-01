//! Constructive Solid Geometry (CSG) operations.
//!
//! This module provides functions for combining shapes using boolean operations:
//!
//! - [`new_intersection`]: Creates a shape that is the intersection of multiple shapes
//! - [`new_difference`]: Creates a shape that subtracts shapes from the first one
//!
//! # Example
//!
//! ```no_run
//! use ln::{new_intersection, new_difference, Sphere, Cube, Vector, Scene, Shape};
//! use std::sync::Arc;
//!
//! // Create a sphere-cube intersection minus a smaller sphere
//! let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 1.0));
//! let cube: Arc<dyn Shape + Send + Sync> = Arc::new(Cube::new(
//!     Vector::new(-0.8, -0.8, -0.8),
//!     Vector::new(0.8, 0.8, 0.8),
//! ));
//! let small_sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 0.5));
//!
//! // (Sphere ∩ Cube) - SmallSphere
//! let shape = new_difference(vec![
//!     new_intersection(vec![sphere, cube]),
//!     small_sphere,
//! ]);
//!
//! let mut scene = Scene::new();
//! scene.add_arc(shape);
//! ```

use crate::bounding_box::Box;
use crate::filter::Filter;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{EmptyShape, Shape};
use crate::vector::Vector;
use std::sync::Arc;

/// Boolean operation type for CSG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Intersection: keeps only the volume that is inside both shapes.
    Intersection,
    /// Difference: subtracts the second shape from the first.
    Difference,
}

/// A shape created by combining two shapes with a boolean operation.
pub struct BooleanShape {
    /// The operation to perform.
    pub op: Op,
    /// The first operand shape.
    pub a: Arc<dyn Shape + Send + Sync>,
    /// The second operand shape.
    pub b: Arc<dyn Shape + Send + Sync>,
}

impl BooleanShape {
    /// Creates a new boolean shape.
    pub fn new(op: Op, a: Arc<dyn Shape + Send + Sync>, b: Arc<dyn Shape + Send + Sync>) -> Self {
        BooleanShape { op, a, b }
    }
}

/// Creates a boolean shape from multiple shapes.
///
/// The shapes are combined pairwise using the given operation.
pub fn new_boolean_shape(
    op: Op,
    shapes: Vec<Arc<dyn Shape + Send + Sync>>,
) -> Arc<dyn Shape + Send + Sync> {
    if shapes.is_empty() {
        return Arc::new(EmptyShape);
    }
    let mut shape: Arc<dyn Shape + Send + Sync> = shapes[0].clone();
    for s in shapes.into_iter().skip(1) {
        shape = Arc::new(BooleanShape::new(op, shape, s));
    }
    shape
}

/// Creates an intersection of multiple shapes.
///
/// The resulting shape contains only the volume that is inside all input shapes.
///
/// # Example
///
/// ```
/// use ln::{new_intersection, Sphere, Cube, Vector, Shape};
/// use std::sync::Arc;
///
/// let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 1.0));
/// let cube: Arc<dyn Shape + Send + Sync> = Arc::new(Cube::new(
///     Vector::new(-0.8, -0.8, -0.8),
///     Vector::new(0.8, 0.8, 0.8),
/// ));
///
/// let intersection = new_intersection(vec![sphere, cube]);
/// ```
pub fn new_intersection(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Arc<dyn Shape + Send + Sync> {
    new_boolean_shape(Op::Intersection, shapes)
}

/// Creates a difference of shapes.
///
/// The resulting shape is the first shape minus all subsequent shapes.
///
/// # Example
///
/// ```
/// use ln::{new_difference, Sphere, Cube, Vector, Shape};
/// use std::sync::Arc;
///
/// let cube: Arc<dyn Shape + Send + Sync> = Arc::new(Cube::new(
///     Vector::new(-1.0, -1.0, -1.0),
///     Vector::new(1.0, 1.0, 1.0),
/// ));
/// let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 0.5));
///
/// // Cube with a spherical hole
/// let difference = new_difference(vec![cube, sphere]);
/// ```
pub fn new_difference(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Arc<dyn Shape + Send + Sync> {
    new_boolean_shape(Op::Difference, shapes)
}

impl Shape for BooleanShape {
    fn compile(&mut self) {
        if let Some(s) = Arc::get_mut(&mut self.a) {
            s.compile();
        }
        if let Some(s) = Arc::get_mut(&mut self.b) {
            s.compile();
        }
    }

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

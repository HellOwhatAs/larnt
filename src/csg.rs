//! Constructive Solid Geometry (CSG) operations.
//!
//! This module provides functions for combining shapes using boolean operations:
//!
//! - [`new_intersection`]: Creates a shape that is the intersection of multiple shapes
//! - [`new_difference`]: Creates a shape that subtracts shapes from the first one
//!
//! # Example
//!
//! ```
//! use larnt::{Cube, Primitive, Sphere, Vector, new_difference, new_intersection};
//!
//! // Create a sphere-cube intersection minus a smaller sphere
//! let sphere: Primitive = Sphere::builder(Vector::default(), 1.0).build().into();
//! let cube: Primitive =
//!     Cube::builder(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8)).build().into();
//! let small_sphere: Primitive = Sphere::builder(Vector::default(), 0.5).build().into();
//! // (Sphere ∩ Cube) - SmallSphere
//! let _shape: Primitive = new_difference(vec![new_intersection(vec![sphere, cube]), small_sphere]);
//! ```

use crate::bounding_box::BBox;
use crate::filter::Filter;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{EmptyShape, RenderArgs, Shape};
use crate::vector::Vector;

/// Boolean operation type for CSG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Intersection: keeps only the volume that is inside both shapes.
    Intersection,
    /// Difference: subtracts the second shape from the first.
    Difference,
}

/// A shape created by combining two shapes with a boolean operation.
#[derive(Debug, Clone)]
pub struct BooleanShape<T> {
    /// The operation to perform.
    pub op: Op,
    /// The first operand shape.
    pub a: Box<T>,
    /// The second operand shape.
    pub b: Box<T>,
}

impl<T: Shape> BooleanShape<T> {
    /// Creates a new boolean shape.
    pub fn new(op: Op, a: Box<T>, b: Box<T>) -> Self {
        BooleanShape { op, a, b }
    }
}

/// Creates a boolean shape from multiple shapes.
///
/// The shapes are combined pairwise using the given operation.
pub fn new_boolean_shape<T>(op: Op, shapes: Vec<T>) -> T
where
    T: Shape + From<BooleanShape<T>> + From<EmptyShape>,
{
    shapes
        .into_iter()
        .reduce(|acc, s| BooleanShape::new(op, Box::new(acc), Box::new(s)).into())
        .unwrap_or_else(|| EmptyShape.into())
}

/// Creates an intersection of multiple shapes.
///
/// The resulting shape contains only the volume that is inside all input shapes.
///
/// # Example
///
/// ```
/// use larnt::{Cube, Primitive, Sphere, Vector, new_intersection};
///
/// let sphere: Primitive = Sphere::builder(Vector::default(), 1.0).build().into();
/// let cube: Primitive =
///     Cube::builder(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8)).build().into();
///
/// let _intersection: Primitive = new_intersection(vec![sphere, cube]);
/// ```
pub fn new_intersection<T>(shapes: Vec<T>) -> T
where
    T: Shape + From<BooleanShape<T>> + From<EmptyShape>,
{
    new_boolean_shape(Op::Intersection, shapes)
}

/// Creates a difference of shapes.
///
/// The resulting shape is the first shape minus all subsequent shapes.
///
/// # Example
///
/// ```
///  use larnt::{Cube, Primitive, Sphere, Vector, new_difference};
///
///  let cube: Primitive =
///      Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build().into();
///  let sphere: Primitive = Sphere::builder(Vector::default(), 0.5).build().into();
///
///  // Cube with a spherical hole
///  let _difference: Primitive = new_difference(vec![cube, sphere]);
/// ```
pub fn new_difference<T>(shapes: Vec<T>) -> T
where
    T: Shape + From<BooleanShape<T>> + From<EmptyShape>,
{
    new_boolean_shape(Op::Difference, shapes)
}

impl<T: Shape> Shape for BooleanShape<T> {
    fn bounding_box(&self) -> BBox {
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

    fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
        let mut p = self.a.paths(args);
        p.extend(self.b.paths(args));
        p = p.chop_adaptive(args);
        p = p.filter(self);
        p
    }
}

impl<T: Shape> Filter for BooleanShape<T> {
    fn filter(&self, v: Vector) -> Option<Vector> {
        if self.contains(v, 0.0) { Some(v) } else { None }
    }
}

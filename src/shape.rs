//! Shape trait and transformations.
//!
//! This module defines the core [`Shape`] trait that all renderable geometry
//! must implement, along with utilities like [`TransformedShape`] for applying
//! transformations and [`EmptyShape`] as a placeholder.
//!
//! # Implementing Custom Shapes
//!
//! To create a custom shape, implement the [`Shape`] trait:
//!
//! ```ignore
//! use ln::{Shape, Paths, Vector, Box, Hit, Ray};
//!
//! struct MySphere {
//!     center: Vector,
//!     radius: f64,
//! }
//!
//! impl Shape for MySphere {
//!     fn bounding_box(&self) -> Box {
//!         Box::new(
//!             self.center.sub_scalar(self.radius),
//!             self.center.add_scalar(self.radius),
//!         )
//!     }
//!
//!     fn contains(&self, v: Vector, f: f64) -> bool {
//!         v.sub(self.center).length() <= self.radius + f
//!     }
//!
//!     fn intersect(&self, r: Ray) -> Hit {
//!         // Ray-sphere intersection logic
//!         Hit::no_hit()
//!     }
//!
//!     fn paths(&self) -> Paths {
//!         // Return paths that represent this shape's surface
//!         Paths::new()
//!     }
//! }
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::vector::Vector;

/// The core trait for all renderable 3D geometry.
///
/// Any type implementing `Shape` can be added to a [`Scene`](crate::Scene) and rendered.
/// The trait requires methods for bounding box computation, containment testing,
/// ray intersection, and path generation.
///
/// # Required Methods
///
/// - [`bounding_box`](Shape::bounding_box): Returns the axis-aligned bounding box
/// - [`contains`](Shape::contains): Tests if a point is inside the solid
/// - [`intersect`](Shape::intersect): Tests for ray-solid intersection
/// - [`paths`](Shape::paths): Returns the 3D paths to render
///
/// # Optional Methods
///
/// - [`compile`](Shape::compile): Perform any preprocessing (default: no-op)
pub trait Shape {
    /// Performs any preprocessing needed before rendering.
    ///
    /// This is called once before rendering begins. Override this for shapes
    /// that need to build acceleration structures or precompute data.
    fn compile(&mut self) {}

    /// Returns the axis-aligned bounding box of this shape.
    ///
    /// The bounding box is used for spatial partitioning and early-out
    /// intersection tests.
    fn bounding_box(&self) -> Box;

    /// Tests if a point is inside this solid.
    ///
    /// The parameter `f` is a fuzz factor to handle floating-point precision
    /// issues near surfaces. A point within distance `f` of the surface should
    /// be considered inside.
    ///
    /// This method is primarily used for CSG (Constructive Solid Geometry)
    /// operations.
    fn contains(&self, v: Vector, f: f64) -> bool;

    /// Tests for ray-solid intersection.
    ///
    /// Returns a [`Hit`] with the distance to the intersection point, or
    /// [`Hit::no_hit()`] if the ray doesn't intersect this shape.
    fn intersect(&self, r: Ray) -> Hit;

    /// Returns the 3D paths that represent this shape's surface.
    ///
    /// These paths are the visual representation of the shape. For a cube,
    /// this might be the 12 edges. For a sphere, it could be latitude and
    /// longitude lines. Custom implementations can return any pattern.
    fn paths(&self) -> Paths;
}

/// A shape that represents empty space.
///
/// This is useful as a placeholder or for testing. It has an empty bounding
/// box, contains no points, never intersects rays, and produces no paths.
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

/// A shape with a transformation matrix applied.
///
/// `TransformedShape` wraps another shape and applies a transformation matrix
/// to it. This allows you to rotate, scale, and translate shapes without
/// modifying the original shape.
///
/// # Example
///
/// ```
/// use ln::{Cube, Matrix, TransformedShape, Vector, radians};
/// use std::sync::Arc;
///
/// let cube = Arc::new(Cube::new(
///     Vector::new(-1.0, -1.0, -1.0),
///     Vector::new(1.0, 1.0, 1.0),
/// ));
///
/// // Rotate cube 45 degrees around Z axis
/// let transform = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(45.0));
/// let rotated = TransformedShape::new(cube, transform);
/// ```
pub struct TransformedShape {
    /// The underlying shape being transformed.
    pub shape: std::sync::Arc<dyn Shape + Send + Sync>,
    /// The transformation matrix to apply.
    pub matrix: Matrix,
    /// The inverse of the transformation matrix (cached for efficiency).
    pub inverse: Matrix,
}

impl TransformedShape {
    /// Creates a new transformed shape.
    ///
    /// The inverse matrix is computed automatically and cached for use
    /// in intersection and containment tests.
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

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
//! ```no_run
//! use larnt::{BBox, Shape, RenderArgs, Paths, Vector, Hit, Ray};
//!
//! struct MySphere {
//!     center: Vector,
//!     radius: f64,
//! }
//!
//! impl Shape for MySphere {
//!     fn bounding_box(&self) -> BBox {
//!         BBox::new(
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
//!     fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
//!         // Return paths that represent this shape's surface
//!         Paths::new()
//!     }
//! }
//! ```

use crate::bounding_box::BBox;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::vector::Vector;

/// The core trait for all renderable 3D geometry.
///
/// Any type implementing `Shape` can be passed to [`render`](crate::render) and rendered.
/// The trait requires methods for bounding box computation, containment testing,
/// ray intersection, and path generation.
///
/// # Required Methods
///
/// - [`bounding_box`](Shape::bounding_box): Returns the axis-aligned bounding box
/// - [`contains`](Shape::contains): Tests if a point is inside the solid
/// - [`intersect`](Shape::intersect): Tests for ray-solid intersection
/// - [`paths`](Shape::paths): Returns the 3D paths to render
pub trait Shape {
    /// Returns the axis-aligned bounding box of this shape.
    ///
    /// The bounding box is used for spatial partitioning and early-out
    /// intersection tests.
    fn bounding_box(&self) -> BBox;

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
    fn paths(&self, args: &RenderArgs) -> Paths<Vector>;
}

#[derive(Debug, Clone)]
pub struct RenderArgs {
    pub screen_mat: Matrix,
    pub eye: Vector,
    pub up: Vector,
    pub width: f64,
    pub height: f64,
    pub step: f64,
}

/// Automatically implement `Shape` for references to shapes.
impl<T: Shape + ?Sized> Shape for &T {
    fn bounding_box(&self) -> BBox {
        (*self).bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        (*self).contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        (*self).intersect(r)
    }

    fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
        (*self).paths(args)
    }
}

/// A shape that represents empty space.
///
/// This is useful as a placeholder or for testing. It has an empty bounding
/// box, contains no points, never intersects rays, and produces no paths.
#[derive(Debug, Clone, Default)]
pub struct EmptyShape;

impl Shape for EmptyShape {
    fn bounding_box(&self) -> BBox {
        BBox::new(Vector::default(), Vector::default())
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, _r: Ray) -> Hit {
        Hit::no_hit()
    }

    fn paths(&self, _args: &RenderArgs) -> Paths<Vector> {
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
/// use larnt::{Cube, Matrix, TransformedShape, Vector, radians};
///
/// let cube = Cube::builder(
///     Vector::new(-1.0, -1.0, -1.0),
///     Vector::new(1.0, 1.0, 1.0),
/// ).build();
///
/// // Rotate cube 45 degrees around Z axis
/// let transform = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(45.0));
/// let rotated = TransformedShape::new(cube, transform);
/// ```
pub struct TransformedShape<T> {
    /// The underlying shape being transformed.
    pub shape: T,
    /// The transformation matrix to apply.
    pub matrix: Matrix,
    /// The inverse of the transformation matrix (cached for efficiency).
    pub inverse: Matrix,
}

impl<T> TransformedShape<T> {
    /// Creates a new transformed shape.
    ///
    /// The inverse matrix is computed automatically and cached for use
    /// in intersection and containment tests.
    pub fn new(shape: T, matrix: Matrix) -> Self {
        let inverse = matrix.inverse();
        TransformedShape {
            shape,
            matrix,
            inverse,
        }
    }
}

impl<T: Shape> Shape for TransformedShape<T> {
    fn bounding_box(&self) -> BBox {
        self.matrix.mul_box(self.shape.bounding_box())
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.shape.contains(self.inverse.mul_position(v), f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.shape.intersect(self.inverse.mul_ray(r))
    }

    fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
        self.shape
            .paths(&RenderArgs {
                screen_mat: args.screen_mat.mul(&self.matrix),
                eye: self.inverse.mul_position(args.eye),
                up: self.inverse.mul_direction(args.up),
                ..args.clone()
            })
            .transform(&self.matrix)
    }
}

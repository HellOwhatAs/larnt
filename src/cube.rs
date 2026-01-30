//! Axis-aligned cube primitive.
//!
//! This module provides the [`Cube`] shape, which is an axis-aligned box
//! (rectangular cuboid) defined by two opposite corners.
//!
//! # Example
//!
//! ```
//! use ln::{Cube, Scene, Vector};
//!
//! // Create a 2x2x2 cube centered at the origin
//! let cube = Cube::new(
//!     Vector::new(-1.0, -1.0, -1.0),
//!     Vector::new(1.0, 1.0, 1.0),
//! );
//!
//! let mut scene = Scene::new();
//! scene.add(cube);
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::vector::Vector;

/// An axis-aligned cube (rectangular cuboid).
///
/// A `Cube` is defined by two opposite corners (minimum and maximum points).
/// The default paths generated are the 12 edges of the cube.
///
/// # Example
///
/// ```
/// use ln::{Cube, Vector};
///
/// // Unit cube from (0,0,0) to (1,1,1)
/// let cube = Cube::new(Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Cube {
    /// The minimum corner (smallest x, y, z values).
    pub min: Vector,
    /// The maximum corner (largest x, y, z values).
    pub max: Vector,
    /// Cached bounding box.
    pub bx: Box,
}

impl Cube {
    /// Creates a new cube from two opposite corners.
    ///
    /// The corners define the axis-aligned bounding box of the cube.
    /// The `min` corner should have smaller x, y, z values than `max`.
    pub fn new(min: Vector, max: Vector) -> Self {
        Cube {
            min,
            max,
            bx: Box::new(min, max),
        }
    }
}

impl Shape for Cube {
    fn bounding_box(&self) -> Box {
        self.bx
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        if v.x < self.min.x - f || v.x > self.max.x + f {
            return false;
        }
        if v.y < self.min.y - f || v.y > self.max.y + f {
            return false;
        }
        if v.z < self.min.z - f || v.z > self.max.z + f {
            return false;
        }
        true
    }

    fn intersect(&self, r: Ray) -> Hit {
        let n = self.min.sub(r.origin).div(r.direction);
        let f = self.max.sub(r.origin).div(r.direction);
        let (n, f) = (n.min(f), n.max(f));
        let t0 = n.x.max(n.y).max(n.z);
        let t1 = f.x.min(f.y).min(f.z);
        
        if t0 < 1e-3 && t1 > 1e-3 {
            return Hit::new(t1);
        }
        if t0 >= 1e-3 && t0 < t1 {
            return Hit::new(t0);
        }
        Hit::no_hit()
    }

    fn paths(&self) -> Paths {
        let (x1, y1, z1) = (self.min.x, self.min.y, self.min.z);
        let (x2, y2, z2) = (self.max.x, self.max.y, self.max.z);
        
        Paths::from_vec(vec![
            vec![Vector::new(x1, y1, z1), Vector::new(x1, y1, z2)],
            vec![Vector::new(x1, y1, z1), Vector::new(x1, y2, z1)],
            vec![Vector::new(x1, y1, z1), Vector::new(x2, y1, z1)],
            vec![Vector::new(x1, y1, z2), Vector::new(x1, y2, z2)],
            vec![Vector::new(x1, y1, z2), Vector::new(x2, y1, z2)],
            vec![Vector::new(x1, y2, z1), Vector::new(x1, y2, z2)],
            vec![Vector::new(x1, y2, z1), Vector::new(x2, y2, z1)],
            vec![Vector::new(x1, y2, z2), Vector::new(x2, y2, z2)],
            vec![Vector::new(x2, y1, z1), Vector::new(x2, y1, z2)],
            vec![Vector::new(x2, y1, z1), Vector::new(x2, y2, z1)],
            vec![Vector::new(x2, y1, z2), Vector::new(x2, y2, z2)],
            vec![Vector::new(x2, y2, z1), Vector::new(x2, y2, z2)],
        ])
    }
}

//! 3D vector mathematics.
//!
//! This module provides the [`Vector`] struct for 3D vector operations.
//! Vectors are used throughout `ln` for positions, directions, and colors.
//!
//! # Example
//!
//! ```
//! use ln::Vector;
//!
//! let a = Vector::new(1.0, 2.0, 3.0);
//! let b = Vector::new(4.0, 5.0, 6.0);
//!
//! // Basic operations
//! let sum = a.add(b);
//! let diff = a.sub(b);
//! let dot = a.dot(b);
//! let cross = a.cross(b);
//!
//! // Normalize to unit length
//! let normalized = a.normalize();
//! assert!((normalized.length() - 1.0).abs() < 1e-10);
//! ```

use rand::Rng;
use std::ops::{Add, Div, Mul, Sub};

/// A 3D vector with x, y, and z components.
///
/// `Vector` is the fundamental type for 3D mathematics in `ln`. It supports
/// standard vector operations like addition, subtraction, dot product, cross
/// product, and normalization.
///
/// # Operator Overloading
///
/// `Vector` implements `Add`, `Sub`, `Mul`, and `Div` for both vector-vector
/// and vector-scalar operations.
///
/// ```
/// use ln::Vector;
///
/// let a = Vector::new(1.0, 2.0, 3.0);
/// let b = Vector::new(2.0, 3.0, 4.0);
///
/// let sum = a + b;  // Vector addition
/// let scaled = a * 2.0;  // Scalar multiplication
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    /// The x component.
    pub x: f64,
    /// The y component.
    pub y: f64,
    /// The z component.
    pub z: f64,
}

impl Vector {
    /// Creates a new vector with the given components.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector { x, y, z }
    }

    /// Returns a random unit vector (uniformly distributed on the unit sphere).
    pub fn random_unit_vector() -> Self {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen::<f64>() * 2.0 - 1.0;
            let y = rng.gen::<f64>() * 2.0 - 1.0;
            let z = rng.gen::<f64>() * 2.0 - 1.0;
            if x * x + y * y + z * z <= 1.0 {
                return Vector::new(x, y, z).normalize();
            }
        }
    }

    /// Returns the length (magnitude) of this vector.
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the distance between this vector and another.
    pub fn distance(&self, other: Vector) -> f64 {
        self.sub(other).length()
    }

    /// Returns the squared length of this vector.
    ///
    /// This is more efficient than `length()` when you only need to compare
    /// distances, since it avoids the square root.
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the squared distance between this vector and another.
    pub fn distance_squared(&self, other: Vector) -> f64 {
        self.sub(other).length_squared()
    }

    /// Returns the dot product of this vector and another.
    ///
    /// The dot product is useful for computing angles between vectors
    /// and for projection operations.
    pub fn dot(&self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the cross product of this vector and another.
    ///
    /// The cross product produces a vector perpendicular to both inputs.
    /// The magnitude equals the area of the parallelogram formed by the
    /// two vectors.
    pub fn cross(&self, other: Vector) -> Vector {
        Vector {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Normalizes the vector to unit length.
    /// 
    /// # Panics
    /// Panics if the vector has zero length.
    pub fn normalize(&self) -> Vector {
        let d = self.length();
        assert!(d != 0.0, "Cannot normalize a zero-length vector");
        Vector {
            x: self.x / d,
            y: self.y / d,
            z: self.z / d,
        }
    }

    pub fn add(&self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(&self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn mul(&self, other: Vector) -> Vector {
        Vector {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    pub fn div(&self, other: Vector) -> Vector {
        Vector {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }

    pub fn add_scalar(&self, s: f64) -> Vector {
        Vector {
            x: self.x + s,
            y: self.y + s,
            z: self.z + s,
        }
    }

    pub fn sub_scalar(&self, s: f64) -> Vector {
        Vector {
            x: self.x - s,
            y: self.y - s,
            z: self.z - s,
        }
    }

    pub fn mul_scalar(&self, s: f64) -> Vector {
        Vector {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn div_scalar(&self, s: f64) -> Vector {
        Vector {
            x: self.x / s,
            y: self.y / s,
            z: self.z / s,
        }
    }

    pub fn min(&self, other: Vector) -> Vector {
        Vector {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(&self, other: Vector) -> Vector {
        Vector {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn min_axis(&self) -> Vector {
        let x = self.x.abs();
        let y = self.y.abs();
        let z = self.z.abs();
        if x <= y && x <= z {
            Vector::new(1.0, 0.0, 0.0)
        } else if y <= x && y <= z {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(0.0, 0.0, 1.0)
        }
    }

    pub fn min_component(&self) -> f64 {
        self.x.min(self.y).min(self.z)
    }

    pub fn segment_distance(&self, v: Vector, w: Vector) -> f64 {
        let l2 = v.distance_squared(w);
        if l2 == 0.0 {
            return self.distance(v);
        }
        let t = self.sub(v).dot(w.sub(v)) / l2;
        if t < 0.0 {
            return self.distance(v);
        }
        if t > 1.0 {
            return self.distance(w);
        }
        v.add(w.sub(v).mul_scalar(t)).distance(*self)
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, other: Vector) -> Vector {
        Vector::add(&self, other)
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, other: Vector) -> Vector {
        Vector::sub(&self, other)
    }
}

impl Mul for Vector {
    type Output = Vector;
    fn mul(self, other: Vector) -> Vector {
        Vector::mul(&self, other)
    }
}

impl Div for Vector {
    type Output = Vector;
    fn div(self, other: Vector) -> Vector {
        Vector::div(&self, other)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, scalar: f64) -> Vector {
        self.mul_scalar(scalar)
    }
}

impl Div<f64> for Vector {
    type Output = Vector;
    fn div(self, scalar: f64) -> Vector {
        self.div_scalar(scalar)
    }
}

impl std::hash::Hash for Vector {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl Eq for Vector {}

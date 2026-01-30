//! 4x4 transformation matrices.
//!
//! This module provides the [`Matrix`] struct for 3D transformations including
//! translation, rotation, scaling, and projection.
//!
//! # Example
//!
//! ```
//! use ln::{Matrix, Vector, radians};
//!
//! // Create a rotation matrix (45 degrees around Z axis)
//! let rotation = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(45.0));
//!
//! // Create a translation matrix
//! let translation = Matrix::translate(Vector::new(1.0, 2.0, 3.0));
//!
//! // Combine transformations
//! let combined = rotation.translated(Vector::new(1.0, 2.0, 3.0));
//! ```

use crate::ray::Ray;
use crate::vector::Vector;

/// A 4x4 transformation matrix.
///
/// `Matrix` represents affine and projective transformations in 3D space.
/// It is used for positioning shapes, camera transformations, and projections.
///
/// # Matrix Layout
///
/// The matrix is stored in row-major order:
/// ```text
/// | x00 x01 x02 x03 |
/// | x10 x11 x12 x13 |
/// | x20 x21 x22 x23 |
/// | x30 x31 x32 x33 |
/// ```
///
/// # Example
///
/// ```
/// use ln::{Matrix, Vector, radians};
///
/// // Transform a point
/// let transform = Matrix::translate(Vector::new(1.0, 0.0, 0.0));
/// let point = Vector::new(0.0, 0.0, 0.0);
/// let transformed = transform.mul_position(point);
/// assert!((transformed.x - 1.0).abs() < 1e-10);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Matrix {
    pub x00: f64, pub x01: f64, pub x02: f64, pub x03: f64,
    pub x10: f64, pub x11: f64, pub x12: f64, pub x13: f64,
    pub x20: f64, pub x21: f64, pub x22: f64, pub x23: f64,
    pub x30: f64, pub x31: f64, pub x32: f64, pub x33: f64,
}

impl Matrix {
    /// Returns the 4x4 identity matrix.
    pub fn identity() -> Self {
        Matrix {
            x00: 1.0, x01: 0.0, x02: 0.0, x03: 0.0,
            x10: 0.0, x11: 1.0, x12: 0.0, x13: 0.0,
            x20: 0.0, x21: 0.0, x22: 1.0, x23: 0.0,
            x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0,
        }
    }

    /// Creates a translation matrix.
    ///
    /// # Example
    ///
    /// ```
    /// use ln::{Matrix, Vector};
    ///
    /// let t = Matrix::translate(Vector::new(1.0, 2.0, 3.0));
    /// let p = t.mul_position(Vector::new(0.0, 0.0, 0.0));
    /// assert!((p.x - 1.0).abs() < 1e-10);
    /// ```
    pub fn translate(v: Vector) -> Self {
        Matrix {
            x00: 1.0, x01: 0.0, x02: 0.0, x03: v.x,
            x10: 0.0, x11: 1.0, x12: 0.0, x13: v.y,
            x20: 0.0, x21: 0.0, x22: 1.0, x23: v.z,
            x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0,
        }
    }

    /// Creates a scale matrix.
    pub fn scale(v: Vector) -> Self {
        Matrix {
            x00: v.x, x01: 0.0, x02: 0.0, x03: 0.0,
            x10: 0.0, x11: v.y, x12: 0.0, x13: 0.0,
            x20: 0.0, x21: 0.0, x22: v.z, x23: 0.0,
            x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0,
        }
    }

    /// Creates a rotation matrix.
    ///
    /// Rotates around the axis `v` by angle `a` (in radians).
    ///
    /// # Example
    ///
    /// ```
    /// use ln::{Matrix, Vector, radians};
    ///
    /// // Rotate 90 degrees around Z axis
    /// let r = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(90.0));
    /// ```
    pub fn rotate(v: Vector, a: f64) -> Self {
        let v = v.normalize();
        let s = a.sin();
        let c = a.cos();
        let m = 1.0 - c;
        Matrix {
            x00: m * v.x * v.x + c,
            x01: m * v.x * v.y + v.z * s,
            x02: m * v.z * v.x - v.y * s,
            x03: 0.0,
            x10: m * v.x * v.y - v.z * s,
            x11: m * v.y * v.y + c,
            x12: m * v.y * v.z + v.x * s,
            x13: 0.0,
            x20: m * v.z * v.x + v.y * s,
            x21: m * v.y * v.z - v.x * s,
            x22: m * v.z * v.z + c,
            x23: 0.0,
            x30: 0.0,
            x31: 0.0,
            x32: 0.0,
            x33: 1.0,
        }
    }

    /// Creates a frustum projection matrix.
    pub fn frustum(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) -> Self {
        let t1 = 2.0 * n;
        let t2 = r - l;
        let t3 = t - b;
        let t4 = f - n;
        Matrix {
            x00: t1 / t2, x01: 0.0, x02: (r + l) / t2, x03: 0.0,
            x10: 0.0, x11: t1 / t3, x12: (t + b) / t3, x13: 0.0,
            x20: 0.0, x21: 0.0, x22: (-f - n) / t4, x23: (-t1 * f) / t4,
            x30: 0.0, x31: 0.0, x32: -1.0, x33: 0.0,
        }
    }

    /// Creates an orthographic projection matrix.
    pub fn orthographic(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) -> Self {
        Matrix {
            x00: 2.0 / (r - l), x01: 0.0, x02: 0.0, x03: -(r + l) / (r - l),
            x10: 0.0, x11: 2.0 / (t - b), x12: 0.0, x13: -(t + b) / (t - b),
            x20: 0.0, x21: 0.0, x22: -2.0 / (f - n), x23: -(f + n) / (f - n),
            x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0,
        }
    }

    /// Creates a perspective projection matrix.
    ///
    /// # Arguments
    ///
    /// * `fovy` - Vertical field of view in degrees
    /// * `aspect` - Aspect ratio (width / height)
    /// * `near` - Near clipping plane distance
    /// * `far` - Far clipping plane distance
    pub fn perspective(fovy: f64, aspect: f64, near: f64, far: f64) -> Self {
        let ymax = near * (fovy * std::f64::consts::PI / 360.0).tan();
        let xmax = ymax * aspect;
        Self::frustum(-xmax, xmax, -ymax, ymax, near, far)
    }

    /// Creates a view matrix looking at a target point.
    ///
    /// # Arguments
    ///
    /// * `eye` - Camera position
    /// * `center` - Point to look at
    /// * `up` - Up direction vector
    pub fn look_at(eye: Vector, center: Vector, up: Vector) -> Self {
        let up = up.normalize();
        let f = center.sub(eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f).normalize();
        let m = Matrix {
            x00: s.x, x01: u.x, x02: -f.x, x03: eye.x,
            x10: s.y, x11: u.y, x12: -f.y, x13: eye.y,
            x20: s.z, x21: u.z, x22: -f.z, x23: eye.z,
            x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0,
        };
        m.inverse()
    }

    /// Returns a new matrix with a translation applied.
    pub fn translated(&self, v: Vector) -> Matrix {
        Matrix::translate(v).mul(self)
    }

    /// Returns a new matrix with a scale applied.
    pub fn scaled(&self, v: Vector) -> Matrix {
        Matrix::scale(v).mul(self)
    }

    /// Returns a new matrix with a rotation applied.
    pub fn rotated(&self, v: Vector, a: f64) -> Matrix {
        Matrix::rotate(v, a).mul(self)
    }

    /// Returns a new matrix with a frustum projection applied.
    pub fn with_frustum(&self, l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) -> Matrix {
        Matrix::frustum(l, r, b, t, n, f).mul(self)
    }

    /// Returns a new matrix with an orthographic projection applied.
    pub fn with_orthographic(&self, l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) -> Matrix {
        Matrix::orthographic(l, r, b, t, n, f).mul(self)
    }

    /// Returns a new matrix with a perspective projection applied.
    pub fn with_perspective(&self, fovy: f64, aspect: f64, near: f64, far: f64) -> Matrix {
        Matrix::perspective(fovy, aspect, near, far).mul(self)
    }

    /// Multiplies this matrix by another matrix.
    pub fn mul(&self, b: &Matrix) -> Matrix {
        let a = self;
        Matrix {
            x00: a.x00 * b.x00 + a.x01 * b.x10 + a.x02 * b.x20 + a.x03 * b.x30,
            x10: a.x10 * b.x00 + a.x11 * b.x10 + a.x12 * b.x20 + a.x13 * b.x30,
            x20: a.x20 * b.x00 + a.x21 * b.x10 + a.x22 * b.x20 + a.x23 * b.x30,
            x30: a.x30 * b.x00 + a.x31 * b.x10 + a.x32 * b.x20 + a.x33 * b.x30,
            x01: a.x00 * b.x01 + a.x01 * b.x11 + a.x02 * b.x21 + a.x03 * b.x31,
            x11: a.x10 * b.x01 + a.x11 * b.x11 + a.x12 * b.x21 + a.x13 * b.x31,
            x21: a.x20 * b.x01 + a.x21 * b.x11 + a.x22 * b.x21 + a.x23 * b.x31,
            x31: a.x30 * b.x01 + a.x31 * b.x11 + a.x32 * b.x21 + a.x33 * b.x31,
            x02: a.x00 * b.x02 + a.x01 * b.x12 + a.x02 * b.x22 + a.x03 * b.x32,
            x12: a.x10 * b.x02 + a.x11 * b.x12 + a.x12 * b.x22 + a.x13 * b.x32,
            x22: a.x20 * b.x02 + a.x21 * b.x12 + a.x22 * b.x22 + a.x23 * b.x32,
            x32: a.x30 * b.x02 + a.x31 * b.x12 + a.x32 * b.x22 + a.x33 * b.x32,
            x03: a.x00 * b.x03 + a.x01 * b.x13 + a.x02 * b.x23 + a.x03 * b.x33,
            x13: a.x10 * b.x03 + a.x11 * b.x13 + a.x12 * b.x23 + a.x13 * b.x33,
            x23: a.x20 * b.x03 + a.x21 * b.x13 + a.x22 * b.x23 + a.x23 * b.x33,
            x33: a.x30 * b.x03 + a.x31 * b.x13 + a.x32 * b.x23 + a.x33 * b.x33,
        }
    }

    /// Transforms a position (point) by this matrix.
    pub fn mul_position(&self, b: Vector) -> Vector {
        let x = self.x00 * b.x + self.x01 * b.y + self.x02 * b.z + self.x03;
        let y = self.x10 * b.x + self.x11 * b.y + self.x12 * b.z + self.x13;
        let z = self.x20 * b.x + self.x21 * b.y + self.x22 * b.z + self.x23;
        Vector::new(x, y, z)
    }

    /// Transforms a position with perspective divide.
    pub fn mul_position_w(&self, b: Vector) -> Vector {
        let x = self.x00 * b.x + self.x01 * b.y + self.x02 * b.z + self.x03;
        let y = self.x10 * b.x + self.x11 * b.y + self.x12 * b.z + self.x13;
        let z = self.x20 * b.x + self.x21 * b.y + self.x22 * b.z + self.x23;
        let w = self.x30 * b.x + self.x31 * b.y + self.x32 * b.z + self.x33;
        Vector::new(x / w, y / w, z / w)
    }

    /// Transforms a direction vector by this matrix.
    ///
    /// Unlike `mul_position`, this ignores the translation component
    /// and normalizes the result.
    pub fn mul_direction(&self, b: Vector) -> Vector {
        let x = self.x00 * b.x + self.x01 * b.y + self.x02 * b.z;
        let y = self.x10 * b.x + self.x11 * b.y + self.x12 * b.z;
        let z = self.x20 * b.x + self.x21 * b.y + self.x22 * b.z;
        Vector::new(x, y, z).normalize()
    }

    /// Transforms a ray by this matrix.
    pub fn mul_ray(&self, b: Ray) -> Ray {
        Ray::new(self.mul_position(b.origin), self.mul_direction(b.direction))
    }

    /// Transforms a bounding box by this matrix.
    pub fn mul_box(&self, bx: crate::bounding_box::Box) -> crate::bounding_box::Box {
        let r = Vector::new(self.x00, self.x10, self.x20);
        let u = Vector::new(self.x01, self.x11, self.x21);
        let b = Vector::new(self.x02, self.x12, self.x22);
        let t = Vector::new(self.x03, self.x13, self.x23);
        let xa = r.mul_scalar(bx.min.x);
        let xb = r.mul_scalar(bx.max.x);
        let ya = u.mul_scalar(bx.min.y);
        let yb = u.mul_scalar(bx.max.y);
        let za = b.mul_scalar(bx.min.z);
        let zb = b.mul_scalar(bx.max.z);
        let (xa, xb) = (xa.min(xb), xa.max(xb));
        let (ya, yb) = (ya.min(yb), ya.max(yb));
        let (za, zb) = (za.min(zb), za.max(zb));
        let min = xa.add(ya).add(za).add(t);
        let max = xb.add(yb).add(zb).add(t);
        crate::bounding_box::Box { min, max }
    }

    /// Returns the transpose of this matrix.
    pub fn transpose(&self) -> Matrix {
        Matrix {
            x00: self.x00, x01: self.x10, x02: self.x20, x03: self.x30,
            x10: self.x01, x11: self.x11, x12: self.x21, x13: self.x31,
            x20: self.x02, x21: self.x12, x22: self.x22, x23: self.x32,
            x30: self.x03, x31: self.x13, x32: self.x23, x33: self.x33,
        }
    }

    /// Computes the determinant of this matrix.
    pub fn determinant(&self) -> f64 {
        let a = self;
        a.x00 * a.x11 * a.x22 * a.x33 - a.x00 * a.x11 * a.x23 * a.x32 +
        a.x00 * a.x12 * a.x23 * a.x31 - a.x00 * a.x12 * a.x21 * a.x33 +
        a.x00 * a.x13 * a.x21 * a.x32 - a.x00 * a.x13 * a.x22 * a.x31 -
        a.x01 * a.x12 * a.x23 * a.x30 + a.x01 * a.x12 * a.x20 * a.x33 -
        a.x01 * a.x13 * a.x20 * a.x32 + a.x01 * a.x13 * a.x22 * a.x30 -
        a.x01 * a.x10 * a.x22 * a.x33 + a.x01 * a.x10 * a.x23 * a.x32 +
        a.x02 * a.x13 * a.x20 * a.x31 - a.x02 * a.x13 * a.x21 * a.x30 +
        a.x02 * a.x10 * a.x21 * a.x33 - a.x02 * a.x10 * a.x23 * a.x31 +
        a.x02 * a.x11 * a.x23 * a.x30 - a.x02 * a.x11 * a.x20 * a.x33 -
        a.x03 * a.x10 * a.x21 * a.x32 + a.x03 * a.x10 * a.x22 * a.x31 -
        a.x03 * a.x11 * a.x22 * a.x30 + a.x03 * a.x11 * a.x20 * a.x32 -
        a.x03 * a.x12 * a.x20 * a.x31 + a.x03 * a.x12 * a.x21 * a.x30
    }

    /// Computes the inverse of this matrix.
    pub fn inverse(&self) -> Matrix {
        let a = self;
        let d = self.determinant();
        Matrix {
            x00: (a.x12 * a.x23 * a.x31 - a.x13 * a.x22 * a.x31 + a.x13 * a.x21 * a.x32 - a.x11 * a.x23 * a.x32 - a.x12 * a.x21 * a.x33 + a.x11 * a.x22 * a.x33) / d,
            x01: (a.x03 * a.x22 * a.x31 - a.x02 * a.x23 * a.x31 - a.x03 * a.x21 * a.x32 + a.x01 * a.x23 * a.x32 + a.x02 * a.x21 * a.x33 - a.x01 * a.x22 * a.x33) / d,
            x02: (a.x02 * a.x13 * a.x31 - a.x03 * a.x12 * a.x31 + a.x03 * a.x11 * a.x32 - a.x01 * a.x13 * a.x32 - a.x02 * a.x11 * a.x33 + a.x01 * a.x12 * a.x33) / d,
            x03: (a.x03 * a.x12 * a.x21 - a.x02 * a.x13 * a.x21 - a.x03 * a.x11 * a.x22 + a.x01 * a.x13 * a.x22 + a.x02 * a.x11 * a.x23 - a.x01 * a.x12 * a.x23) / d,
            x10: (a.x13 * a.x22 * a.x30 - a.x12 * a.x23 * a.x30 - a.x13 * a.x20 * a.x32 + a.x10 * a.x23 * a.x32 + a.x12 * a.x20 * a.x33 - a.x10 * a.x22 * a.x33) / d,
            x11: (a.x02 * a.x23 * a.x30 - a.x03 * a.x22 * a.x30 + a.x03 * a.x20 * a.x32 - a.x00 * a.x23 * a.x32 - a.x02 * a.x20 * a.x33 + a.x00 * a.x22 * a.x33) / d,
            x12: (a.x03 * a.x12 * a.x30 - a.x02 * a.x13 * a.x30 - a.x03 * a.x10 * a.x32 + a.x00 * a.x13 * a.x32 + a.x02 * a.x10 * a.x33 - a.x00 * a.x12 * a.x33) / d,
            x13: (a.x02 * a.x13 * a.x20 - a.x03 * a.x12 * a.x20 + a.x03 * a.x10 * a.x22 - a.x00 * a.x13 * a.x22 - a.x02 * a.x10 * a.x23 + a.x00 * a.x12 * a.x23) / d,
            x20: (a.x11 * a.x23 * a.x30 - a.x13 * a.x21 * a.x30 + a.x13 * a.x20 * a.x31 - a.x10 * a.x23 * a.x31 - a.x11 * a.x20 * a.x33 + a.x10 * a.x21 * a.x33) / d,
            x21: (a.x03 * a.x21 * a.x30 - a.x01 * a.x23 * a.x30 - a.x03 * a.x20 * a.x31 + a.x00 * a.x23 * a.x31 + a.x01 * a.x20 * a.x33 - a.x00 * a.x21 * a.x33) / d,
            x22: (a.x01 * a.x13 * a.x30 - a.x03 * a.x11 * a.x30 + a.x03 * a.x10 * a.x31 - a.x00 * a.x13 * a.x31 - a.x01 * a.x10 * a.x33 + a.x00 * a.x11 * a.x33) / d,
            x23: (a.x03 * a.x11 * a.x20 - a.x01 * a.x13 * a.x20 - a.x03 * a.x10 * a.x21 + a.x00 * a.x13 * a.x21 + a.x01 * a.x10 * a.x23 - a.x00 * a.x11 * a.x23) / d,
            x30: (a.x12 * a.x21 * a.x30 - a.x11 * a.x22 * a.x30 - a.x12 * a.x20 * a.x31 + a.x10 * a.x22 * a.x31 + a.x11 * a.x20 * a.x32 - a.x10 * a.x21 * a.x32) / d,
            x31: (a.x01 * a.x22 * a.x30 - a.x02 * a.x21 * a.x30 + a.x02 * a.x20 * a.x31 - a.x00 * a.x22 * a.x31 - a.x01 * a.x20 * a.x32 + a.x00 * a.x21 * a.x32) / d,
            x32: (a.x02 * a.x11 * a.x30 - a.x01 * a.x12 * a.x30 - a.x02 * a.x10 * a.x31 + a.x00 * a.x12 * a.x31 + a.x01 * a.x10 * a.x32 - a.x00 * a.x11 * a.x32) / d,
            x33: (a.x01 * a.x12 * a.x20 - a.x02 * a.x11 * a.x20 + a.x02 * a.x10 * a.x21 - a.x00 * a.x12 * a.x21 - a.x01 * a.x10 * a.x22 + a.x00 * a.x11 * a.x22) / d,
        }
    }
}

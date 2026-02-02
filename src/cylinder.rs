//! Cylinder primitive.
//!
//! This module provides the [`Cylinder`] shape (aligned along the Z axis)
//! and [`OutlineCylinder`] which renders as a silhouette from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use ln::{Cylinder, Scene, Vector};
//!
//! // Create a cylinder with radius 1.0, from z=0 to z=2
//! let cylinder = Cylinder::new(1.0, 0.0, 2.0);
//!
//! let mut scene = Scene::new();
//! scene.add(cylinder);
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{Shape, TransformedShape};
use crate::util::radians;
use crate::vector::Vector;
use std::sync::Arc;

/// A cylinder aligned along the Z axis.
///
/// The cylinder is defined by its radius and Z-range. The default paths
/// are vertical lines around the circumference.
///
/// # Example
///
/// ```
/// use ln::{Cylinder, Vector};
///
/// // Cylinder with radius 0.5, from z=-1 to z=1
/// let cylinder = Cylinder::new(0.5, -1.0, 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct Cylinder {
    /// The radius of the cylinder.
    pub radius: f64,
    /// The minimum Z coordinate.
    pub z0: f64,
    /// The maximum Z coordinate.
    pub z1: f64,
}

impl Cylinder {
    /// Creates a new cylinder with the given radius and Z-range.
    pub fn new(radius: f64, z0: f64, z1: f64) -> Self {
        Cylinder { radius, z0, z1 }
    }
}

impl Shape for Cylinder {
    fn bounding_box(&self) -> Box {
        let r = self.radius;
        Box::new(Vector::new(-r, -r, self.z0), Vector::new(r, r, self.z1))
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        let xy = Vector::new(v.x, v.y, 0.0);
        if xy.length() > self.radius + f {
            return false;
        }
        v.z >= self.z0 - f && v.z <= self.z1 + f
    }

    fn intersect(&self, ray: Ray) -> Hit {
        let r = self.radius;
        let o = ray.origin;
        let d = ray.direction;
        let a = d.x * d.x + d.y * d.y;
        let b = 2.0 * o.x * d.x + 2.0 * o.y * d.y;
        let c = o.x * o.x + o.y * o.y - r * r;
        let q = b * b - 4.0 * a * c;

        if q < 0.0 {
            return Hit::no_hit();
        }

        let s = q.sqrt();
        let mut t0 = (-b + s) / (2.0 * a);
        let mut t1 = (-b - s) / (2.0 * a);

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let z0 = o.z + t0 * d.z;
        let z1 = o.z + t1 * d.z;

        if t0 > 1e-6 && self.z0 < z0 && z0 < self.z1 {
            return Hit::new(t0);
        }
        if t1 > 1e-6 && self.z0 < z1 && z1 < self.z1 {
            return Hit::new(t1);
        }
        Hit::no_hit()
    }

    fn paths(&self) -> Paths {
        let mut result = Vec::new();
        let mut a = 0;
        while a < 360 {
            let x = self.radius * radians(a as f64).cos();
            let y = self.radius * radians(a as f64).sin();
            result.push(vec![Vector::new(x, y, self.z0), Vector::new(x, y, self.z1)]);
            a += 10;
        }
        Paths::from_vec(result)
    }
}

/// A cylinder that renders as a silhouette from the camera's perspective.
///
/// Unlike [`Cylinder`] which draws vertical lines, `OutlineCylinder` draws
/// only the visible outline of the cylinder as seen from the camera.
#[derive(Debug, Clone)]
pub struct OutlineCylinder {
    /// The underlying cylinder geometry.
    pub cylinder: Cylinder,
    /// The camera position.
    pub eye: Vector,
    /// The up direction.
    pub up: Vector,
}

impl OutlineCylinder {
    /// Creates a new outline cylinder.
    pub fn new(eye: Vector, up: Vector, radius: f64, z0: f64, z1: f64) -> Self {
        OutlineCylinder {
            cylinder: Cylinder::new(radius, z0, z1),
            eye,
            up,
        }
    }
}

impl Shape for OutlineCylinder {
    fn bounding_box(&self) -> Box {
        self.cylinder.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.cylinder.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.cylinder.intersect(r)
    }

    fn paths(&self) -> Paths {
        let center = Vector::new(0.0, 0.0, self.cylinder.z0);
        let hyp = center.sub(self.eye).length();
        let opp = self.cylinder.radius;
        let theta = (opp / hyp).asin();
        let adj = opp / theta.tan();
        let d = theta.cos() * adj;
        let w = center.sub(self.eye).normalize();
        let u = w.cross(self.up).normalize();
        let c0 = self.eye.add(w.mul_scalar(d));
        let a0 = c0.add(u.mul_scalar(self.cylinder.radius * 1.01));
        let b0 = c0.add(u.mul_scalar(-self.cylinder.radius * 1.01));

        let center = Vector::new(0.0, 0.0, self.cylinder.z1);
        let hyp = center.sub(self.eye).length();
        let opp = self.cylinder.radius;
        let theta = (opp / hyp).asin();
        let adj = opp / theta.tan();
        let d = theta.cos() * adj;
        let w = center.sub(self.eye).normalize();
        let u = w.cross(self.up).normalize();
        let c1 = self.eye.add(w.mul_scalar(d));
        let a1 = c1.add(u.mul_scalar(self.cylinder.radius * 1.01));
        let b1 = c1.add(u.mul_scalar(-self.cylinder.radius * 1.01));

        let mut p0 = Vec::new();
        let mut p1 = Vec::new();
        for a in 0..360 {
            let x = self.cylinder.radius * radians(a as f64).cos();
            let y = self.cylinder.radius * radians(a as f64).sin();
            p0.push(Vector::new(x, y, self.cylinder.z0));
            p1.push(Vector::new(x, y, self.cylinder.z1));
        }

        Paths::from_vec(vec![
            p0,
            p1,
            vec![
                Vector::new(a0.x, a0.y, self.cylinder.z0),
                Vector::new(a1.x, a1.y, self.cylinder.z1),
            ],
            vec![
                Vector::new(b0.x, b0.y, self.cylinder.z0),
                Vector::new(b1.x, b1.y, self.cylinder.z1),
            ],
        ])
    }
}

pub fn new_transformed_cylinder(
    up: Vector,
    v0: Vector,
    v1: Vector,
    radius: f64,
) -> TransformedShape {
    let d = v1.sub(v0);
    let z = d.length();
    let a = d.normalize().dot(up).acos();
    let m = if a != 0.0 {
        let u = d.cross(up).normalize();
        Matrix::rotate(u, a).translated(v0)
    } else {
        Matrix::translate(v0)
    };
    let c = Cylinder::new(radius, 0.0, z);
    TransformedShape::new(Arc::new(c), m)
}

/// Creates an outline cylinder between two arbitrary points.
///
/// This is useful for drawing cylinders that aren't aligned with the Z axis.
/// The cylinder is created along the axis from `v0` to `v1` with the given radius.
///
/// # Arguments
///
/// * `eye` - Camera position for silhouette calculation
/// * `up` - Up direction vector
/// * `v0` - Start point of the cylinder
/// * `v1` - End point of the cylinder
/// * `radius` - Radius of the cylinder
pub fn new_transformed_outline_cylinder(
    eye: Vector,
    up: Vector,
    v0: Vector,
    v1: Vector,
    radius: f64,
) -> TransformedShape {
    let d = v1.sub(v0);
    let z = d.length();
    let a = d.normalize().dot(up).acos();
    let m = if a != 0.0 {
        let u = d.cross(up).normalize();
        Matrix::rotate(u, a).translated(v0)
    } else {
        Matrix::translate(v0)
    };
    let c = OutlineCylinder::new(m.inverse().mul_position(eye), up, radius, 0.0, z);
    TransformedShape::new(Arc::new(c), m)
}

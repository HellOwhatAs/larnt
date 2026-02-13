//! Cylinder primitive.
//!
//! This module provides the [`Cylinder`] shape (aligned along the Z axis)
//! and [`OutlineCylinder`] which renders as a silhouette from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use larnt::{Cylinder, Scene, Vector};
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
use crate::shape::{RenderArgs, Shape, TransformedShape};
use crate::util::radians;
use crate::vector::Vector;
use std::sync::Arc;

/// Texture options for cylinders.
#[derive(Debug, Clone, Copy, Default)]
pub enum CylinderTexture {
    #[default]
    Outline,
    Striped(u64),
}

/// A cylinder aligned along the Z axis.
///
/// The cylinder is defined by its radius and Z-range. The default paths
/// are vertical lines around the circumference.
///
/// # Example
///
/// ```
/// use larnt::{Cylinder, Vector};
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
    /// The texture style for the cylinder.
    pub texture: CylinderTexture,
}

impl Cylinder {
    /// Creates a new cylinder with the given radius and Z-range.
    pub fn new(radius: f64, z0: f64, z1: f64) -> Self {
        Cylinder {
            radius,
            z0,
            z1,
            texture: CylinderTexture::default(),
        }
    }

    /// Sets the texture style for the cylinder.
    pub fn with_texture(mut self, texture: CylinderTexture) -> Self {
        self.texture = texture;
        self
    }

    fn paths_striped(&self, num: u64) -> Paths {
        let mut result = Vec::new();
        for a in (0..360).step_by((360 / num) as usize) {
            let x = self.radius * radians(a as f64).cos();
            let y = self.radius * radians(a as f64).sin();
            result.push(vec![Vector::new(x, y, self.z0), Vector::new(x, y, self.z1)]);
        }
        Paths::from_vec(result)
    }

    fn paths_outline(&self, args: &RenderArgs) -> Paths {
        // For a cylinder with radius r aligned along Z-axis, the silhouette
        // generators are found by solving:
        // E.x * cos(θ) + E.y * sin(θ) = r
        // where E is the eye position.
        //
        // This is of the form: a*cos(θ) + b*sin(θ) = c
        // Solution: θ = atan2(b, a) ± acos(c / sqrt(a^2 + b^2))
        let r = self.radius;

        let a = args.eye.x;
        let b = args.eye.y;
        let c = r;

        let sqrt_ab = (a * a + b * b).sqrt();

        // Compute silhouette generator angles
        let ratio = c / sqrt_ab;
        if ratio.abs() > 1.0 {
            // Eye is inside the cylinder - no proper silhouette
            // Fall back to full circles
            let mut p0 = Vec::new();
            let mut p1 = Vec::new();
            for angle in 0..=360 {
                let x = r * radians(angle as f64).cos();
                let y = r * radians(angle as f64).sin();
                p0.push(Vector::new(x, y, self.z0));
                p1.push(Vector::new(x, y, self.z1));
            }
            return Paths::from_vec(vec![p0, p1]);
        }

        let eye_azimuth = b.atan2(a);
        let angular_offset = ratio.acos();
        let theta1 = eye_azimuth + angular_offset;
        let theta2 = eye_azimuth - angular_offset;

        // For visibility of arcs, scale outer edge by 1/cos(π/360)
        let vscale = |angle_r: f64| {
            if (angle_r - eye_azimuth).cos() >= ratio {
                1.0 / (std::f64::consts::PI / 360.0).cos()
            } else {
                1.0
            }
        };
        // Top circle
        let mut p1 = Vec::new();
        for angle in 0..=360 {
            let angle_r = radians(angle as f64);
            let x = r * vscale(angle_r) * angle_r.cos();
            let y = r * vscale(angle_r) * angle_r.sin();
            p1.push(Vector::new(x, y, self.z1));
        }

        // Bottom circle
        let mut p0 = Vec::new();
        for angle in 0..=360 {
            let angle_r = radians(angle as f64);
            let x = r * vscale(angle_r) * angle_r.cos();
            let y = r * vscale(angle_r) * angle_r.sin();
            p0.push(Vector::new(x, y, self.z0));
        }

        // Silhouette lines from tangent points
        let a0 = Vector::new(r * theta1.cos(), r * theta1.sin(), self.z0);
        let a1 = Vector::new(r * theta1.cos(), r * theta1.sin(), self.z1);
        let b0 = Vector::new(r * theta2.cos(), r * theta2.sin(), self.z0);
        let b1 = Vector::new(r * theta2.cos(), r * theta2.sin(), self.z1);

        Paths::from_vec(vec![p0, p1, vec![a0, a1], vec![b0, b1]])
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

    fn paths(&self, args: &RenderArgs) -> Paths {
        match self.texture {
            CylinderTexture::Outline => self.paths_outline(args),
            CylinderTexture::Striped(num) => self.paths_striped(num),
        }
    }
}

/// Creates an cylinder between two arbitrary points.
///
/// This is useful for drawing cylinders that aren't aligned with the Z axis.
/// The cylinder is created along the axis from `v0` to `v1` with the given radius.
///
/// # Arguments
///
/// * `v0` - Start point of the cylinder
/// * `v1` - End point of the cylinder
/// * `radius` - Radius of the cylinder
/// * `texture` - Texture style for the cylinder
pub fn new_transformed_cylinder(
    v0: Vector,
    v1: Vector,
    radius: f64,
    texture: CylinderTexture,
) -> TransformedShape {
    let up = Vector::new(0.0, 0.0, 1.0);
    let d = v1.sub(v0);
    let z = d.length();
    let a = d.normalize().dot(up).acos();
    let m = if a != 0.0 {
        let u = d.cross(up).normalize();
        Matrix::rotate(u, a).translated(v0)
    } else {
        Matrix::translate(v0)
    };
    let c = Cylinder::new(radius, 0.0, z).with_texture(texture);
    TransformedShape::new(Arc::new(c), m)
}

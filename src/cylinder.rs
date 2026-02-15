//! Cylinder primitive.
//!
//! This module provides the [`Cylinder`] shape (aligned along the Z axis)
//! and the default [`CylinderTexture`] [`CylinderTexture::Outline`] renders a silhouette
//! from the camera's perspective.
//!
//! # Example
//!
//! ```
//! use larnt::{Cylinder, Scene, Vector};
//!
//! // Create a cylinder with radius 1.0, from z=0 to z=2
//! let cylinder = Cylinder::builder(1.0, 0.0, 2.0).build();
//!
//! let mut scene = Scene::new();
//! scene.add(cylinder);
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::{Paths, adaptive_arc, adaptive_arc_inner};
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape, TransformedShape};
use crate::util::radians;
use crate::vector::Vector;
use bon::{Builder, bon, builder};
use std::f64::consts::PI;
use std::sync::Arc;

/// Texture options for cylinders.
#[derive(Debug, Clone, Copy, Default)]
pub enum CylinderTexture {
    #[default]
    Outline,
    Striped(u64),
}

#[bon]
impl CylinderTexture {
    #[builder]
    pub fn outline() -> Self {
        CylinderTexture::Outline
    }

    #[builder]
    pub fn striped(#[builder(default = 36)] num: u64) -> Self {
        CylinderTexture::Striped(num)
    }
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
/// let cylinder = Cylinder::builder(0.5, -1.0, 1.0).build();
/// ```
#[derive(Debug, Clone, Builder)]
pub struct Cylinder {
    /// The radius of the cylinder.
    #[builder(start_fn)]
    pub radius: f64,
    /// The minimum Z coordinate.
    #[builder(start_fn)]
    pub z0: f64,
    /// The maximum Z coordinate.
    #[builder(start_fn)]
    pub z1: f64,
    /// The texture style for the cylinder.
    #[builder(default)]
    pub texture: CylinderTexture,
}

impl Cylinder {
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

        let (u, v) = (Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));
        let step_sq = args.step.powi(2);

        // Compute silhouette generator angles
        let ratio = c / sqrt_ab;
        if ratio.abs() > 1.0 {
            // Eye is inside the cylinder - no proper silhouette
            // Fall back to full circles
            return Paths::from_vec(
                [self.z0, self.z1]
                    .into_iter()
                    .map(|z| {
                        adaptive_arc_inner(
                            0.0,
                            PI * 2.0,
                            r,
                            &(Vector::new(0.0, 0.0, z), u, v),
                            &args.screen_mat,
                            step_sq,
                        )
                    })
                    .collect(),
            );
        }

        let eye_azimuth = b.atan2(a);
        let angular_offset = ratio.acos();
        let theta1 = eye_azimuth + angular_offset;
        let theta2 = eye_azimuth - angular_offset;

        // Front and back arcs seperately to pass visibility tests
        let mut paths = [adaptive_arc, adaptive_arc_inner]
            .iter()
            .zip([(theta2, theta1), (theta1, theta2 + PI * 2.0)])
            .flat_map(|(func, (alpha, beta))| {
                [self.z0, self.z1].into_iter().map(move |z| {
                    func(
                        alpha,
                        beta,
                        r,
                        &(Vector::new(0.0, 0.0, z), u, v),
                        &args.screen_mat,
                        step_sq,
                    )
                })
            })
            .collect::<Vec<_>>();

        // Silhouette lines from tangent points
        let a0 = Vector::new(r * theta1.cos(), r * theta1.sin(), self.z0);
        let a1 = Vector::new(r * theta1.cos(), r * theta1.sin(), self.z1);
        let b0 = Vector::new(r * theta2.cos(), r * theta2.sin(), self.z0);
        let b1 = Vector::new(r * theta2.cos(), r * theta2.sin(), self.z1);
        paths.push(vec![a0, a1]);
        paths.push(vec![b0, b1]);

        Paths::from_vec(paths)
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
#[builder]
pub fn new_transformed_cylinder(
    #[builder(start_fn)] v0: Vector,
    #[builder(start_fn)] v1: Vector,
    #[builder(start_fn)] radius: f64,
    #[builder(default)] texture: CylinderTexture,
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
    let c = Cylinder::builder(radius, 0.0, z).texture(texture).build();
    TransformedShape::new(Arc::new(c), m)
}

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape, TransformedShape};
use crate::util::radians;
use crate::vector::Vector;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub enum ConeTexture {
    #[default]
    Outline,
    Striped(u64),
}

/// A 3D cone shape defined by a circular base and an apex point.
/// The cone is oriented along the z-axis, with the base centered at
/// the origin (0,0,0) and the apex at (0,0,height).
#[derive(Debug, Clone)]
pub struct Cone {
    pub radius: f64,
    pub height: f64,
    pub texture: ConeTexture,
}

impl Cone {
    pub fn new(radius: f64, height: f64) -> Self {
        Cone {
            radius,
            height,
            texture: ConeTexture::default(),
        }
    }

    /// Sets the texture style for the cone.
    pub fn with_texture(mut self, texture: ConeTexture) -> Self {
        self.texture = texture;
        self
    }

    fn paths_striped(&self, num: u64) -> Paths {
        let mut result = Vec::new();
        for a in (0..360).step_by((360 / num) as usize) {
            let x = self.radius * radians(a as f64).cos();
            let y = self.radius * radians(a as f64).sin();
            result.push(vec![
                Vector::new(x, y, 0.0),
                Vector::new(0.0, 0.0, self.height),
            ]);
        }
        Paths::from_vec(result)
    }

    fn paths_outline(&self, args: &RenderArgs) -> Paths {
        // For a cone with apex at (0,0,h) and base circle radius r at z=0,
        // the silhouette generators are found by solving:
        // E.x * cos(θ) + E.y * sin(θ) = r * (1 - E.z / h)
        // where E is the eye position.
        //
        // This is of the form: a*cos(θ) + b*sin(θ) = c
        // Solution: θ = atan2(b, a) ± acos(c / sqrt(a^2 + b^2))
        let r = self.radius;
        let h = self.height;

        let a = args.eye.x;
        let b = args.eye.y;
        let c = r * (1.0 - args.eye.z / h);

        let sqrt_ab = (a * a + b * b).sqrt();

        // For visibility of arcs, scale by 1/cos(π/360)
        let vscale = 1.0 / (std::f64::consts::PI / 360.0).cos();

        // Compute silhouette generator angles
        let ratio = c / sqrt_ab;
        if ratio.abs() > 1.0 {
            // Eye is inside the extended cone surface - no proper silhouette
            // Fall back to just the base circle
            let mut p0 = Vec::new();
            let vscale = if args.eye.z >= 0.0 && args.eye.z <= h {
                1.0
            } else {
                vscale
            };
            for angle in 0..=360 {
                let x = r * vscale * radians(angle as f64).cos();
                let y = r * vscale * radians(angle as f64).sin();
                p0.push(Vector::new(x, y, 0.0));
            }
            return Paths::from_vec(vec![p0]);
        }

        let eye_azimuth = b.atan2(a);
        let angular_offset = ratio.acos();
        let theta1 = eye_azimuth + angular_offset;
        let theta2 = eye_azimuth - angular_offset;

        // Base circle path
        let mut p0 = Vec::new();
        for angle in 0..=360 {
            let angle_r = radians(angle as f64);
            let x = r * vscale * angle_r.cos();
            let y = r * vscale * angle_r.sin();
            p0.push(Vector::new(x, y, 0.0));
        }

        // Silhouette points on the base circle (with slight outward offset for visibility)
        let a0 = Vector::new(r * theta1.cos(), r * theta1.sin(), 0.0);
        let b0 = Vector::new(r * theta2.cos(), r * theta2.sin(), 0.0);

        Paths::from_vec(vec![
            p0,
            vec![a0, Vector::new(0.0, 0.0, h)],
            vec![b0, Vector::new(0.0, 0.0, h)],
        ])
    }
}

impl Shape for Cone {
    fn bounding_box(&self) -> Box {
        let r = self.radius;
        Box::new(Vector::new(-r, -r, 0.0), Vector::new(r, r, self.height))
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, ray: Ray) -> Hit {
        let o = ray.origin;
        let d = ray.direction;
        let r = self.radius;
        let h = self.height;

        let k = r / h;
        let k = k * k;

        let a = d.x * d.x + d.y * d.y - k * d.z * d.z;
        let b = 2.0 * (d.x * o.x + d.y * o.y - k * d.z * (o.z - h));
        let c = o.x * o.x + o.y * o.y - k * (o.z - h) * (o.z - h);
        let q = b * b - 4.0 * a * c;

        if q <= 0.0 {
            return Hit::no_hit();
        }

        let s = q.sqrt();
        let mut t0 = (-b + s) / (2.0 * a);
        let mut t1 = (-b - s) / (2.0 * a);

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        if t0 > 1e-6 {
            let p = ray.position(t0);
            if p.z > 0.0 && p.z < h {
                return Hit::new(t0);
            }
        }
        if t1 > 1e-6 {
            let p = ray.position(t1);
            if p.z > 0.0 && p.z < h {
                return Hit::new(t1);
            }
        }
        Hit::no_hit()
    }

    fn paths(&self, args: &RenderArgs) -> Paths {
        match self.texture {
            ConeTexture::Outline => self.paths_outline(args),
            ConeTexture::Striped(num) => self.paths_striped(num),
        }
    }
}

pub fn new_transformed_cone(
    v0: Vector,
    v1: Vector,
    radius: f64,
    texture: ConeTexture,
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
    let c = Cone::new(radius, z).with_texture(texture);
    TransformedShape::new(Arc::new(c), m)
}

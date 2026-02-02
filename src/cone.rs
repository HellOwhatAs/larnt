use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{Shape, TransformedShape};
use crate::util::radians;
use crate::vector::Vector;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Cone {
    pub radius: f64,
    pub height: f64,
}

impl Cone {
    pub fn new(radius: f64, height: f64) -> Self {
        Cone { radius, height }
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

    fn paths(&self) -> Paths {
        let mut result = Vec::new();
        let mut a = 0;
        while a < 360 {
            let x = self.radius * radians(a as f64).cos();
            let y = self.radius * radians(a as f64).sin();
            result.push(vec![
                Vector::new(x, y, 0.0),
                Vector::new(0.0, 0.0, self.height),
            ]);
            a += 30;
        }
        Paths::from_vec(result)
    }
}

#[derive(Debug, Clone)]
pub struct OutlineCone {
    pub cone: Cone,
    pub eye: Vector,
    pub up: Vector,
}

impl OutlineCone {
    pub fn new(eye: Vector, up: Vector, radius: f64, height: f64) -> Self {
        OutlineCone {
            cone: Cone::new(radius, height),
            eye,
            up,
        }
    }
}

impl Shape for OutlineCone {
    fn bounding_box(&self) -> Box {
        self.cone.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.cone.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.cone.intersect(r)
    }

    fn paths(&self) -> Paths {
        // For a cone with apex at (0,0,h) and base circle radius r at z=0,
        // the silhouette generators are found by solving:
        // E.x * cos(θ) + E.y * sin(θ) = r * (1 - E.z / h)
        // where E is the eye position.
        //
        // This is of the form: a*cos(θ) + b*sin(θ) = c
        // Solution: θ = atan2(b, a) ± acos(c / sqrt(a^2 + b^2))
        let r = self.cone.radius;
        let h = self.cone.height;

        let a = self.eye.x;
        let b = self.eye.y;
        let c = r * (1.0 - self.eye.z / h);

        let sqrt_ab = (a * a + b * b).sqrt();

        // Base circle path
        let mut p0 = Vec::new();
        for angle in 0..360 {
            let x = r * radians(angle as f64).cos();
            let y = r * radians(angle as f64).sin();
            p0.push(Vector::new(x, y, 0.0));
        }

        // Compute silhouette generator angles
        let ratio = c / sqrt_ab;
        if ratio.abs() > 1.0 {
            // Eye is inside the extended cone surface - no proper silhouette
            // Fall back to just the base circle
            return Paths::from_vec(vec![p0]);
        }

        let eye_azimuth = b.atan2(a);
        let angular_offset = ratio.acos();
        let theta1 = eye_azimuth + angular_offset;
        let theta2 = eye_azimuth - angular_offset;

        // Silhouette points on the base circle (with slight outward offset for visibility)
        let scale = 1.01;
        let a0 = Vector::new(r * scale * theta1.cos(), r * scale * theta1.sin(), 0.0);
        let b0 = Vector::new(r * scale * theta2.cos(), r * scale * theta2.sin(), 0.0);

        Paths::from_vec(vec![
            p0,
            vec![a0, Vector::new(0.0, 0.0, h)],
            vec![b0, Vector::new(0.0, 0.0, h)],
        ])
    }
}

pub fn new_transformed_cone(up: Vector, v0: Vector, v1: Vector, radius: f64) -> TransformedShape {
    let d = v1.sub(v0);
    let z = d.length();
    let a = d.normalize().dot(up).acos();
    let m = if a != 0.0 {
        let u = d.cross(up).normalize();
        Matrix::rotate(u, a).translated(v0)
    } else {
        Matrix::translate(v0)
    };
    let c = Cone::new(radius, z);
    TransformedShape::new(Arc::new(c), m)
}

pub fn new_transformed_outline_cone(
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
    let c = OutlineCone::new(m.inverse().mul_position(eye), up, radius, z);
    TransformedShape::new(Arc::new(c), m)
}

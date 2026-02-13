use crate::bounding_box::Box;
use crate::common::EPS;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::vector::Vector;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v1: Vector,
    pub v2: Vector,
    pub v3: Vector,
    pub bx: Box,
}

impl Triangle {
    pub fn new(v1: Vector, v2: Vector, v3: Vector) -> Self {
        Self {
            v1,
            v2,
            v3,
            bx: Box::new(v1.min(v2).min(v3), v1.max(v2).max(v3)),
        }
    }

    pub fn intersect_vertices(v1: Vector, v2: Vector, v3: Vector, r: Ray) -> Hit {
        let e1x = v2.x - v1.x;
        let e1y = v2.y - v1.y;
        let e1z = v2.z - v1.z;
        let e2x = v3.x - v1.x;
        let e2y = v3.y - v1.y;
        let e2z = v3.z - v1.z;
        let px = r.direction.y * e2z - r.direction.z * e2y;
        let py = r.direction.z * e2x - r.direction.x * e2z;
        let pz = r.direction.x * e2y - r.direction.y * e2x;
        let det = e1x * px + e1y * py + e1z * pz;

        if det > -EPS && det < EPS {
            return Hit::no_hit();
        }

        let inv = 1.0 / det;
        let tx = r.origin.x - v1.x;
        let ty = r.origin.y - v1.y;
        let tz = r.origin.z - v1.z;
        let u = (tx * px + ty * py + tz * pz) * inv;

        if u < 0.0 || u > 1.0 {
            return Hit::no_hit();
        }

        let qx = ty * e1z - tz * e1y;
        let qy = tz * e1x - tx * e1z;
        let qz = tx * e1y - ty * e1x;
        let v = (r.direction.x * qx + r.direction.y * qy + r.direction.z * qz) * inv;

        if v < 0.0 || u + v > 1.0 {
            return Hit::no_hit();
        }

        let d = (e2x * qx + e2y * qy + e2z * qz) * inv;

        if d < EPS {
            return Hit::no_hit();
        }

        Hit::new(d)
    }
}

impl Shape for Triangle {
    fn bounding_box(&self) -> Box {
        self.bx
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, r: Ray) -> Hit {
        Self::intersect_vertices(self.v1, self.v2, self.v3, r)
    }

    fn paths(&self, _args: &RenderArgs) -> Paths {
        Paths::from_vec(vec![
            vec![self.v1, self.v2],
            vec![self.v2, self.v3],
            vec![self.v3, self.v1],
        ])
    }
}

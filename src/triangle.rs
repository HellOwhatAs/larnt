use crate::bounding_box::Box;
use crate::common::EPS;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
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
        let mut t = Triangle {
            v1, v2, v3,
            bx: Box::default(),
        };
        t.update_bounding_box();
        t
    }

    pub fn update_bounding_box(&mut self) {
        let min = self.v1.min(self.v2).min(self.v3);
        let max = self.v1.max(self.v2).max(self.v3);
        self.bx = Box::new(min, max);
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
        let e1x = self.v2.x - self.v1.x;
        let e1y = self.v2.y - self.v1.y;
        let e1z = self.v2.z - self.v1.z;
        let e2x = self.v3.x - self.v1.x;
        let e2y = self.v3.y - self.v1.y;
        let e2z = self.v3.z - self.v1.z;
        let px = r.direction.y * e2z - r.direction.z * e2y;
        let py = r.direction.z * e2x - r.direction.x * e2z;
        let pz = r.direction.x * e2y - r.direction.y * e2x;
        let det = e1x * px + e1y * py + e1z * pz;
        
        if det > -EPS && det < EPS {
            return Hit::no_hit();
        }
        
        let inv = 1.0 / det;
        let tx = r.origin.x - self.v1.x;
        let ty = r.origin.y - self.v1.y;
        let tz = r.origin.z - self.v1.z;
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

    fn paths(&self) -> Paths {
        Paths::from_vec(vec![
            vec![self.v1, self.v2],
            vec![self.v2, self.v3],
            vec![self.v3, self.v1],
        ])
    }
}

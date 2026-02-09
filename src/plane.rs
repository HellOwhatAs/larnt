use crate::common::EPS;
use crate::mesh::Mesh;
use crate::path::Paths;
use crate::triangle::Triangle;
use crate::vector::Vector;

#[derive(Debug, Clone)]
pub struct Plane {
    pub point: Vector,
    pub normal: Vector,
}

impl Plane {
    pub fn new(point: Vector, normal: Vector) -> Self {
        Plane { point, normal }
    }

    pub fn intersect_segment(&self, v0: Vector, v1: Vector) -> Option<Vector> {
        let u = v1.sub(v0);
        let w = v0.sub(self.point);
        let d = self.normal.dot(u);
        let n = -self.normal.dot(w);

        if d > -EPS && d < EPS {
            return None;
        }

        let t = n / d;
        if t < 0.0 || t > 1.0 {
            return None;
        }

        Some(v0.add(u.mul_scalar(t)))
    }

    pub fn intersect_triangle(&self, t: &Triangle) -> Option<(Vector, Vector)> {
        let v1 = self.intersect_segment(t.v1, t.v2);
        let v2 = self.intersect_segment(t.v2, t.v3);
        let v3 = self.intersect_segment(t.v3, t.v1);

        match (v1, v2, v3) {
            (Some(v1), Some(v2), _) => Some((v1, v2)),
            (Some(v1), _, Some(v3)) => Some((v1, v3)),
            (_, Some(v2), Some(v3)) => Some((v2, v3)),
            _ => None,
        }
    }

    pub fn intersect_mesh(&self, m: &Mesh) -> Paths {
        let mut result = Vec::new();
        for t in &m.triangles {
            if let Some((v1, v2)) = self.intersect_triangle(t) {
                result.push(vec![v1, v2]);
            }
        }
        Paths::from_vec(result)
    }
}

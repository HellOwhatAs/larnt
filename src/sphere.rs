use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vector,
    pub radius: f64,
    pub bx: Box,
}

impl Sphere {
    pub fn new(center: Vector, radius: f64) -> Self {
        let min = Vector::new(center.x - radius, center.y - radius, center.z - radius);
        let max = Vector::new(center.x + radius, center.y + radius, center.z + radius);
        Sphere {
            center,
            radius,
            bx: Box::new(min, max),
        }
    }
}

impl Shape for Sphere {
    fn bounding_box(&self) -> Box {
        self.bx
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        v.sub(self.center).length() <= self.radius + f
    }

    fn intersect(&self, r: Ray) -> Hit {
        let radius = self.radius;
        let to = r.origin.sub(self.center);
        let b = to.dot(r.direction);
        let c = to.dot(to) - radius * radius;
        let d = b * b - c;
        
        if d > 0.0 {
            let d = d.sqrt();
            let t1 = -b - d;
            if t1 > 1e-2 {
                return Hit::new(t1);
            }
            let t2 = -b + d;
            if t2 > 1e-2 {
                return Hit::new(t2);
            }
        }
        Hit::no_hit()
    }

    fn paths(&self) -> Paths {
        let mut paths = Vec::new();
        let n = 10;
        let o = 10;
        
        // Latitude lines
        let mut lat = -90 + o;
        while lat <= 90 - o {
            let mut path = Vec::new();
            for lng in 0..=360 {
                let v = lat_lng_to_xyz(lat as f64, lng as f64, self.radius).add(self.center);
                path.push(v);
            }
            paths.push(path);
            lat += n;
        }
        
        // Longitude lines
        let mut lng = 0;
        while lng <= 360 {
            let mut path = Vec::new();
            for lat in (-90 + o)..=(90 - o) {
                let v = lat_lng_to_xyz(lat as f64, lng as f64, self.radius).add(self.center);
                path.push(v);
            }
            paths.push(path);
            lng += n;
        }
        
        Paths::from_vec(paths)
    }
}

pub fn lat_lng_to_xyz(lat: f64, lng: f64, radius: f64) -> Vector {
    let lat = radians(lat);
    let lng = radians(lng);
    let x = radius * lat.cos() * lng.cos();
    let y = radius * lat.cos() * lng.sin();
    let z = radius * lat.sin();
    Vector::new(x, y, z)
}

#[derive(Debug, Clone)]
pub struct OutlineSphere {
    pub sphere: Sphere,
    pub eye: Vector,
    pub up: Vector,
}

impl OutlineSphere {
    pub fn new(eye: Vector, up: Vector, center: Vector, radius: f64) -> Self {
        OutlineSphere {
            sphere: Sphere::new(center, radius),
            eye,
            up,
        }
    }
}

impl Shape for OutlineSphere {
    fn bounding_box(&self) -> Box {
        self.sphere.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.sphere.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.sphere.intersect(r)
    }

    fn paths(&self) -> Paths {
        let center = self.sphere.center;
        let radius = self.sphere.radius;
        
        let hyp = center.sub(self.eye).length();
        let opp = radius;
        let theta = (opp / hyp).asin();
        let adj = opp / theta.tan();
        let d = theta.cos() * adj;
        let r = theta.sin() * adj;
        
        let w = center.sub(self.eye).normalize();
        let u = w.cross(self.up).normalize();
        let v = w.cross(u).normalize();
        let c = self.eye.add(w.mul_scalar(d));
        
        let mut path = Vec::new();
        for i in 0..=360 {
            let a = radians(i as f64);
            let mut p = c;
            p = p.add(u.mul_scalar(a.cos() * r));
            p = p.add(v.mul_scalar(a.sin() * r));
            path.push(p);
        }
        
        Paths::from_vec(vec![path])
    }
}

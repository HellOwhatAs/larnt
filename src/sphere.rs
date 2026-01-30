//! Sphere primitive.
//!
//! This module provides the [`Sphere`] shape with latitude/longitude line texturing,
//! and [`OutlineSphere`] which renders as a silhouette circle from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use ln::{Scene, Sphere, Vector};
//!
//! // Create a unit sphere at the origin
//! let sphere = Sphere::new(Vector::new(0.0, 0.0, 0.0), 1.0);
//!
//! let mut scene = Scene::new();
//! scene.add(sphere);
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;

/// A sphere defined by center and radius.
///
/// The default paths generated are latitude and longitude lines, creating
/// a globe-like appearance.
///
/// # Example
///
/// ```
/// use ln::{Sphere, Vector};
///
/// // Sphere at origin with radius 2
/// let sphere = Sphere::new(Vector::new(0.0, 0.0, 0.0), 2.0);
/// ```
#[derive(Debug, Clone)]
pub struct Sphere {
    /// The center point of the sphere.
    pub center: Vector,
    /// The radius of the sphere.
    pub radius: f64,
    /// Cached bounding box.
    pub bx: Box,
}

impl Sphere {
    /// Creates a new sphere with the given center and radius.
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

/// Converts latitude and longitude to 3D coordinates on a sphere.
///
/// # Arguments
///
/// * `lat` - Latitude in degrees (-90 to 90)
/// * `lng` - Longitude in degrees (0 to 360)
/// * `radius` - Radius of the sphere
///
/// # Returns
///
/// A [`Vector`] representing the point on the sphere surface.
pub fn lat_lng_to_xyz(lat: f64, lng: f64, radius: f64) -> Vector {
    let lat = radians(lat);
    let lng = radians(lng);
    let x = radius * lat.cos() * lng.cos();
    let y = radius * lat.cos() * lng.sin();
    let z = radius * lat.sin();
    Vector::new(x, y, z)
}

/// A sphere that renders as a silhouette circle from the camera's perspective.
///
/// Unlike [`Sphere`] which draws latitude/longitude lines, `OutlineSphere`
/// draws only the visible outline of the sphere as seen from the camera.
/// This is useful for cleaner, more stylized renderings.
///
/// # Example
///
/// ```
/// use ln::{OutlineSphere, Scene, Vector};
///
/// let eye = Vector::new(4.0, 3.0, 2.0);
/// let up = Vector::new(0.0, 0.0, 1.0);
///
/// let sphere = OutlineSphere::new(eye, up, Vector::new(0.0, 0.0, 0.0), 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct OutlineSphere {
    /// The underlying sphere geometry.
    pub sphere: Sphere,
    /// The camera position (used to compute the silhouette).
    pub eye: Vector,
    /// The up direction (used to orient the silhouette).
    pub up: Vector,
}

impl OutlineSphere {
    /// Creates a new outline sphere.
    ///
    /// # Arguments
    ///
    /// * `eye` - The camera position
    /// * `up` - The up direction vector
    /// * `center` - The center of the sphere
    /// * `radius` - The radius of the sphere
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
        
        // Handle case when w is parallel to up vector by finding a perpendicular vector
        let cross = w.cross(self.up);
        let u = if cross.length_squared() < 1e-18 {
            // w is parallel to up, use the minimum axis approach to find a perpendicular
            w.cross(w.min_axis()).normalize()
        } else {
            cross.normalize()
        };
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

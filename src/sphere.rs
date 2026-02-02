//! Sphere primitive.
//!
//! This module provides the [`Sphere`] shape with multiple texture options,
//! and [`OutlineSphere`] which renders as a silhouette circle from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use ln::{Scene, Sphere, SphereTexture, Vector};
//!
//! // Create a unit sphere at the origin with the default lat/lng texture
//! let sphere = Sphere::new(Vector::new(0.0, 0.0, 0.0), 1.0);
//!
//! // Or with a custom texture
//! let sphere_dots = Sphere::new(Vector::new(2.0, 0.0, 0.0), 1.0)
//!     .with_texture(SphereTexture::RandomDots(42));
//!
//! let mut scene = Scene::new();
//! scene.add(sphere);
//! scene.add(sphere_dots);
//! ```

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;
use rand::{rngs::SmallRng, Rng, SeedableRng};

/// Texture style for Sphere shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SphereTexture {
    /// Latitude/longitude grid texture (default)
    #[default]
    LatLng,
    /// Random rotated equators (great circles)
    RandomEquators(u64),
    /// Random point dots on the surface
    RandomDots(u64),
    /// Random concentric circles pattern
    RandomCircles(u64),
}

/// A sphere defined by center and radius.
///
/// The default paths generated are latitude and longitude lines, creating
/// a globe-like appearance. You can use [`with_texture`](Sphere::with_texture)
/// to select different texture styles.
///
/// # Example
///
/// ```
/// use ln::{Sphere, SphereTexture, Vector};
///
/// // Sphere at origin with radius 2 (default lat/lng texture)
/// let sphere = Sphere::new(Vector::new(0.0, 0.0, 0.0), 2.0);
///
/// // Sphere with dots texture
/// let sphere_dots = Sphere::new(Vector::new(0.0, 0.0, 0.0), 2.0)
///     .with_texture(SphereTexture::RandomDots(42));
/// ```
#[derive(Debug, Clone)]
pub struct Sphere {
    /// The center point of the sphere.
    pub center: Vector,
    /// The radius of the sphere.
    pub radius: f64,
    /// Cached bounding box.
    pub bx: Box,
    /// The texture style for the sphere.
    pub texture: SphereTexture,
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
            texture: SphereTexture::default(),
        }
    }

    /// Sets the texture style for the sphere.
    pub fn with_texture(mut self, texture: SphereTexture) -> Self {
        self.texture = texture;
        self
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
        match self.texture {
            SphereTexture::LatLng => self.paths_lat_lng(),
            SphereTexture::RandomEquators(seed) => self.paths_random_equators(seed),
            SphereTexture::RandomDots(seed) => self.paths_random_dots(seed),
            SphereTexture::RandomCircles(seed) => self.paths_random_circles(seed),
        }
    }
}

impl Sphere {
    /// Latitude/longitude grid texture (default)
    fn paths_lat_lng(&self) -> Paths {
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
        while lng < 360 {
            let mut path = Vec::new();
            for lat in -(90 - o)..=(90 - o) {
                let v = lat_lng_to_xyz(lat as f64, lng as f64, self.radius).add(self.center);
                path.push(v);
            }
            paths.push(path);
            lng += n;
        }

        Paths::from_vec(paths)
    }

    /// Random rotated equators (great circles)
    fn paths_random_equators(&self, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        // Create a single equator path
        let mut equator = Vec::new();
        for lng in 0..=360 {
            let v = lat_lng_to_xyz(0.0, lng as f64, self.radius);
            equator.push(v);
        }

        let mut paths = Vec::new();
        for _ in 0..100 {
            let mut m = Matrix::identity();
            for _ in 0..3 {
                let v = Vector::random_unit_vector(&mut rng);
                m = m.rotated(v, rng.gen::<f64>() * 2.0 * std::f64::consts::PI);
            }
            m = m.translated(self.center);

            // Transform the equator path
            let transformed: Vec<Vector> = equator.iter().map(|p| m.mul_position(*p)).collect();
            paths.push(transformed);
        }

        Paths::from_vec(paths)
    }

    /// Random point dots on the surface
    fn paths_random_dots(&self, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut paths = Vec::new();

        for _ in 0..20000 {
            let v = Vector::random_unit_vector(&mut rng)
                .mul_scalar(self.radius)
                .add(self.center);
            // Each "dot" is a zero-length path (two identical points)
            paths.push(vec![v, v]);
        }

        Paths::from_vec(paths)
    }

    /// Random concentric circles pattern
    fn paths_random_circles(&self, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut paths = Vec::new();
        let mut seen: Vec<Vector> = Vec::new();
        let mut radii: Vec<f64> = Vec::new();

        for _ in 0..140 {
            let mut v: Vector;
            let mut m: f64;

            // Find a spot that doesn't overlap too much with existing circles
            loop {
                v = Vector::random_unit_vector(&mut rng);
                m = rng.gen::<f64>() * 0.25 + 0.05;

                let mut ok = true;
                for (i, other) in seen.iter().enumerate() {
                    let threshold = m + radii[i] + 0.02;
                    if other.sub(v).length() < threshold {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    seen.push(v);
                    radii.push(m);
                    break;
                }
            }

            // Calculate perpendicular vectors for the circle plane
            let p = v.cross(Vector::random_unit_vector(&mut rng)).normalize();
            let q = p.cross(v).normalize();

            // Draw n concentric circles, each smaller than the last
            let n = rng.gen_range(1..=4);
            let mut current_m = m;
            for _ in 0..n {
                let mut path = Vec::new();
                for j in (0..=360).step_by(5) {
                    let a = radians(j as f64);
                    let mut x = v;
                    x = x.add(p.mul_scalar(a.cos() * current_m));
                    x = x.add(q.mul_scalar(a.sin() * current_m));
                    x = x.normalize();
                    x = x.mul_scalar(self.radius).add(self.center);
                    path.push(x);
                }
                paths.push(path);
                current_m *= 0.75;
            }
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

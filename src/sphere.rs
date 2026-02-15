//! Sphere primitive.
//!
//! This module provides the [`Sphere`] shape with multiple texture options,
//! the default outline texture renders as a silhouette circle from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use larnt::{Scene, Sphere, SphereTexture, Vector};
//!
//! // Create a unit sphere at the origin with the default outline texture
//! let sphere = Sphere::builder(Vector::new(0.0, 0.0, 0.0), 1.0).build();
//!
//! // Or with a custom texture
//! let sphere_fuzz = Sphere::builder(Vector::new(2.0, 0.0, 0.0), 1.0)
//!     .texture(SphereTexture::random_fuzz(42).call())
//!     .build();
//!
//! let mut scene = Scene::new();
//! scene.add(sphere);
//! scene.add(sphere_fuzz);
//! ```

use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::path::adaptive_arc;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;
use crate::{bounding_box::Box, shape::RenderArgs};
use bon::{Builder, bon};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::f64::consts::PI;

/// Texture style for Sphere shapes
#[derive(Debug, Clone, Copy, Default)]
pub enum SphereTexture {
    /// A sphere that renders as a silhouette circle from the camera's perspective.
    #[default]
    Outline,
    /// Latitude/longitude grid texture (default n: 10, o: 10)
    LatLng { n: i32, o: i32 },
    /// Random rotated equators (great circles) (default n: 100)
    RandomEquators { seed: u64, n: usize },
    /// Random fuzz on the surface (default num: 1000, scale: 1.1)
    RandomFuzz { seed: u64, num: usize, scale: f64 },
    /// Random concentric circles pattern (default num: 140)
    RandomCircles { seed: u64, num: usize },
}

#[bon]
impl SphereTexture {
    /// Create a sphere with the default outline texture.
    #[builder]
    pub fn outline() -> Self {
        SphereTexture::Outline
    }

    /// Create a latitude/longitude grid texture with the specified number of lines and offset.
    #[builder]
    pub fn lat_lng(#[builder(default = 10)] n: i32, #[builder(default = 10)] o: i32) -> Self {
        SphereTexture::LatLng { n, o }
    }

    /// Create a random equators texture with the specified number of great circles.
    #[builder]
    pub fn random_equators(
        #[builder(start_fn)] seed: u64,
        #[builder(default = 100)] n: usize,
    ) -> Self {
        SphereTexture::RandomEquators { seed, n }
    }

    /// Create a random fuzz texture with the specified number of points and scale.
    #[builder]
    pub fn random_fuzz(
        #[builder(start_fn)] seed: u64,
        #[builder(default = 1000)] num: usize,
        #[builder(default = 1.1)] scale: f64,
    ) -> Self {
        SphereTexture::RandomFuzz { seed, num, scale }
    }

    /// Create a random concentric circles texture with the specified number of circles.
    #[builder]
    pub fn random_circles(
        #[builder(start_fn)] seed: u64,
        #[builder(default = 140)] num: usize,
    ) -> Self {
        SphereTexture::RandomCircles { seed, num }
    }
}

/// A sphere defined by center and radius.
///
/// The default paths generated are a silhouette circle from the camera's perspective.
///
/// # Example
///
/// ```
/// use larnt::{Sphere, SphereTexture, Vector};
///
/// // Sphere at origin with radius 2 (default outline texture)
/// let sphere = Sphere::builder(Vector::new(0.0, 0.0, 0.0), 2.0).build();
///
/// // Sphere with fuzz texture
/// let sphere_fuzz = Sphere::builder(Vector::new(0.0, 0.0, 0.0), 2.0)
///     .texture(SphereTexture::random_fuzz(42).call())
///     .build();
/// ```
#[derive(Debug, Clone, Builder)]
pub struct Sphere {
    /// The center point of the sphere.
    #[builder(start_fn)]
    pub center: Vector,
    /// The radius of the sphere.
    #[builder(start_fn)]
    pub radius: f64,
    /// Cached bounding box.
    #[builder(skip = {
        let min = Vector::new(center.x - radius, center.y - radius, center.z - radius);
        let max = Vector::new(center.x + radius, center.y + radius, center.z + radius);
        Box::new(min, max)
    })]
    pub bx: Box,
    /// The texture style for the sphere.
    #[builder(default)]
    pub texture: SphereTexture,
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

    fn paths(&self, args: &RenderArgs) -> Paths {
        match self.texture {
            SphereTexture::Outline => self.paths_outline(args),
            SphereTexture::LatLng { n, o } => self.paths_lat_lng(&args.screen_mat, args.step, n, o),
            SphereTexture::RandomEquators { seed, n } => {
                self.paths_random_equators(&args.screen_mat, args.step, n, seed)
            }
            SphereTexture::RandomFuzz { seed, num, scale } => {
                self.paths_random_fuzz(num, scale, seed)
            }
            SphereTexture::RandomCircles { seed, num } => {
                self.paths_random_circles(&args.screen_mat, args.step, num, seed)
            }
        }
    }
}

impl Sphere {
    /// Outline texture: renders as a silhouette circle from the camera's perspective.
    fn paths_outline(&self, args: &RenderArgs) -> Paths {
        let center = self.center;
        let radius = self.radius;

        let hyp = center.sub(args.eye).length();
        let opp = radius;
        if hyp < opp {
            return Paths::new();
        }
        let theta = (opp / hyp).asin();
        let adj = opp / theta.tan();
        let d = theta.cos() * adj;
        let r = theta.sin() * adj;

        let w = center.sub(args.eye).normalize();

        // Handle case when w is parallel to up vector by finding a perpendicular vector
        let cross = w.cross(args.up);
        let u = if cross.length_squared() < 1e-18 {
            // w is parallel to up, use the minimum axis approach to find a perpendicular
            w.cross(w.min_axis()).normalize()
        } else {
            cross.normalize()
        };
        let v = w.cross(u).normalize();
        let c = args.eye.add(w.mul_scalar(d));

        let path = adaptive_arc(
            0.0,
            PI * 2.,
            r,
            &(c, u, v),
            &args.screen_mat,
            args.step.powi(2),
        );
        Paths::from_vec(vec![path])
    }

    /// Latitude/longitude grid texture
    fn paths_lat_lng(&self, screen_mat: &Matrix, step: f64, n: i32, o: i32) -> Paths {
        let mut paths = Vec::new();
        let step_sq = step.powi(2);

        // Latitude lines
        {
            let mut lat = -90 + o;
            while lat <= 90 - o {
                let (c, r) = {
                    let latr = radians(lat as f64);
                    let mut c = self.center;
                    c.z += self.radius * latr.sin();
                    let r = self.radius * latr.cos();
                    (c, r)
                };
                let (u, v) = (Vector::new(1., 0., 0.), Vector::new(0., 1., 0.));

                let path = adaptive_arc(0.0, PI * 2.0, r, &(c, u, v), screen_mat, step_sq);
                paths.push(path);
                lat += n;
            }
        }

        // Longitude lines
        {
            let mut lng = 0;
            let u = Vector::new(0.0, 0.0, 1.0);
            while lng < 360 {
                let (c, r) = (self.center, self.radius);
                let v = {
                    let lngr = radians(lng as f64);
                    Vector::new(lngr.cos(), lngr.sin(), 0.0)
                };
                let [alpha, beta] = [o, 180 - o].map(|x| radians(x as f64));

                let path = adaptive_arc(alpha, beta, r, &(c, u, v), screen_mat, step_sq);
                paths.push(path);
                lng += n;
            }
        }

        Paths::from_vec(paths)
    }

    /// Random rotated equators (great circles)
    fn paths_random_equators(&self, screen_mat: &Matrix, step: f64, n: usize, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        let step_sq = step.powi(2);
        let (c, r) = (self.center, self.radius);

        let mut paths = Vec::with_capacity(n);
        for _ in 0..n {
            let (u, v) = {
                let [u, w] = [(); 2].map(|_| Vector::random_unit_vector(&mut rng));
                (u, w.cross(u).normalize())
            };

            let path = adaptive_arc(0.0, PI * 2.0, r, &(c, u, v), screen_mat, step_sq);
            paths.push(path);
        }

        Paths::from_vec(paths)
    }

    /// Random point dots on the surface
    fn paths_random_fuzz(&self, num: usize, scale: f64, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut paths = Vec::new();

        for _ in 0..num {
            let v = Vector::random_unit_vector(&mut rng);
            paths.push(vec![
                v.mul_scalar(self.radius).add(self.center),
                v.mul_scalar(self.radius * scale).add(self.center),
            ]);
        }

        Paths::from_vec(paths)
    }

    /// Random concentric circles pattern
    fn paths_random_circles(&self, screen_mat: &Matrix, step: f64, num: usize, seed: u64) -> Paths {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut paths = Vec::new();
        let mut seen: Vec<Vector> = Vec::new();
        let mut radii: Vec<f64> = Vec::new();
        let step_sq = step.powi(2);

        for _ in 0..num {
            let mut v: Vector;
            let mut m: f64;

            // Find a spot that doesn't overlap too much with existing circles
            loop {
                v = Vector::random_unit_vector(&mut rng);
                m = rng.random::<f64>() * 0.25 + 0.05;

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
            let n = rng.random_range(1..=4);
            let mut current_m = m;
            for _ in 0..n {
                let (r, c) = {
                    let norm = (v.length_squared() + current_m.powi(2)).sqrt();
                    let r = current_m * self.radius / norm;
                    let c = v.mul_scalar(self.radius / norm).add(self.center);
                    (r, c)
                };

                let path = adaptive_arc(0.0, PI * 2.0, r, &(c, p, q), screen_mat, step_sq);
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

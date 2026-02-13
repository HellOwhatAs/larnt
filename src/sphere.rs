//! Sphere primitive.
//!
//! This module provides the [`Sphere`] shape with multiple texture options,
//! and [`OutlineSphere`] which renders as a silhouette circle from the camera's
//! perspective.
//!
//! # Example
//!
//! ```
//! use larnt::{Scene, Sphere, SphereTexture, Vector};
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

use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;
use crate::{bounding_box::Box, shape::RenderArgs};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::f64::consts::PI;

/// Texture style for Sphere shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SphereTexture {
    /// Latitude/longitude grid texture (default)
    #[default]
    LatLng,
    /// Random rotated equators (great circles)
    RandomEquators(u64),
    /// Random fuzz on the surface
    RandomFuzz(u64),
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
/// use larnt::{Sphere, SphereTexture, Vector};
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

    fn paths(&self, args: &RenderArgs) -> Paths {
        match self.texture {
            SphereTexture::LatLng => self.paths_lat_lng(&args.screen_mat, args.step, 10, 10),
            SphereTexture::RandomEquators(seed) => {
                self.paths_random_equators(&args.screen_mat, args.step, 100, seed)
            }
            SphereTexture::RandomFuzz(seed) => self.paths_random_fuzz(1000, 1.1, seed),
            SphereTexture::RandomCircles(seed) => {
                self.paths_random_circles(&args.screen_mat, args.step, 140, seed)
            }
        }
    }
}

impl Sphere {
    /// Latitude/longitude grid texture (default)
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
                let p = recursive_arc_subdivide(0.0, PI * 2.0, r, &(c, u, v), screen_mat, step_sq);

                let expanded_radius = radius_expansion(&p, r);
                let path = p
                    .iter()
                    .enumerate()
                    .map(|(i, beta)| {
                        let max_r = expanded_radius[i].max(expanded_radius[i + 1]);
                        c.add(u.mul_scalar(beta.cos() * max_r))
                            .add(v.mul_scalar(beta.sin() * max_r))
                    })
                    .collect();
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
                let p = recursive_arc_subdivide(alpha, beta, r, &(c, u, v), screen_mat, step_sq);
                let expanded_radius = radius_expansion(&p, r);
                let path = p
                    .iter()
                    .enumerate()
                    .map(|(i, beta)| {
                        let max_r = expanded_radius[i].max(expanded_radius[i + 1]);
                        c.add(u.mul_scalar(beta.cos() * max_r))
                            .add(v.mul_scalar(beta.sin() * max_r))
                    })
                    .collect();
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
            let path = recursive_arc_subdivide(0.0, PI * 2.0, r, &(c, u, v), screen_mat, step_sq);

            let expanded_radius = radius_expansion(&path, r);
            paths.push(
                path.iter()
                    .enumerate()
                    .map(|(i, beta)| {
                        let max_r = expanded_radius[i].max(expanded_radius[i + 1]);
                        c.add(u.mul_scalar(beta.cos() * max_r))
                            .add(v.mul_scalar(beta.sin() * max_r))
                    })
                    .collect(),
            );
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
                let path =
                    recursive_arc_subdivide(0.0, PI * 2.0, r, &(c, p, q), screen_mat, step_sq);

                let expanded_radius = radius_expansion(&path, r);
                paths.push(
                    path.iter()
                        .enumerate()
                        .map(|(i, beta)| {
                            let max_r = expanded_radius[i].max(expanded_radius[i + 1]);
                            c.add(p.mul_scalar(beta.cos() * max_r))
                                .add(q.mul_scalar(beta.sin() * max_r))
                        })
                        .collect(),
                );
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
/// use larnt::{OutlineSphere, Scene, Vector};
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
    pub fn new(center: Vector, radius: f64) -> Self {
        OutlineSphere {
            sphere: Sphere::new(center, radius),
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

    fn paths(&self, args: &RenderArgs) -> Paths {
        let center = self.sphere.center;
        let radius = self.sphere.radius;

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
        let path = recursive_arc_subdivide(
            0.0,
            PI * 2.0,
            r,
            &(c, u, v),
            &args.screen_mat,
            args.step.powi(2),
        );
        let expanded_radius = radius_expansion(&path, r);
        Paths::from_vec(vec![
            path.iter()
                .enumerate()
                .map(|(i, beta)| {
                    let max_r = expanded_radius[i].max(expanded_radius[i + 1]);
                    c.add(u.mul_scalar(beta.cos() * max_r))
                        .add(v.mul_scalar(beta.sin() * max_r))
                })
                .collect(),
        ])
    }
}

fn radius_expansion(path: &[f64], r: f64) -> Vec<f64> {
    let mut radius: Vec<f64> = std::iter::once(0.0)
        .chain(path.windows(2).map(|x| {
            let (alpha, beta) = (x[0], x[1]);
            let cos_theta = ((beta - alpha) / 2.0).cos();
            r / cos_theta
        }))
        .collect();
    let (back, front) = (radius.last().copied().unwrap(), radius[1]);
    radius[0] = back;
    radius.push(front);
    radius
}

fn recursive_arc_subdivide(
    alpha: f64,
    beta: f64,
    r: f64,
    cuv: &(Vector, Vector, Vector),
    screen_mat: &Matrix,
    step_sq: f64,
) -> Vec<f64> {
    let screen_view = |x: f64| {
        screen_mat.mul_position_w(
            (cuv.0)
                .add((cuv.1).mul_scalar(x.cos() * r))
                .add((cuv.2).mul_scalar(x.sin() * r)),
        )
    };
    let mut path = vec![alpha];
    crate::path::recursive_subdivide(
        ((alpha, screen_view(alpha)), (beta, screen_view(beta))),
        &|(alpha, _), (beta, _)| {
            let mid = (beta + alpha) / 2.0;
            (mid, screen_view(mid))
        },
        &|(alpha, sa), (beta, sb)| {
            let theta = (beta - alpha) / 2.0;
            theta < PI / 180.0
                || sa.distance_squared(sb) * theta / theta.sin() < step_sq && theta < PI / 3.0
        },
        &mut |(x, _)| path.push(x),
    );
    path
}

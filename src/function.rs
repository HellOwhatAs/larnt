use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::path::{Paths, recursive_subdivide};
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::util::radians;
use crate::vector::Vector;
use bon::Builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Above,
    Below,
}

/// Texture style for Function shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FunctionTexture {
    /// Grid texture with lines along constant x and y (works with any function)
    #[default]
    Grid,
    /// Radial swirl texture (best for functions returning negative values like -1/(x²+y²))
    Swirl,
    /// Spiral path texture (works with any function)
    Spiral,
}

#[derive(Debug, Builder)]
pub struct Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    #[builder(start_fn)]
    pub func: F,
    #[builder(start_fn)]
    pub bx: Box,
    #[builder(default = Direction::Below)]
    pub direction: Direction,
    #[builder(default)]
    pub texture: FunctionTexture,
    #[builder(default = 0.1)]
    pub step: f64,
}

impl<F> Shape for Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    fn bounding_box(&self) -> Box {
        self.bx
    }

    fn contains(&self, v: Vector, _eps: f64) -> bool {
        if self.direction == Direction::Below {
            v.z < (self.func)(v.x, v.y)
        } else {
            v.z > (self.func)(v.x, v.y)
        }
    }

    fn intersect(&self, ray: Ray) -> Hit {
        let n = self.bx.min.sub(ray.origin).div(ray.direction);
        let f = self.bx.max.sub(ray.origin).div(ray.direction);
        let (n, f) = (n.min(f), n.max(f));
        let t0 = n.x.max(n.y).max(n.z);
        let t1 = f.x.min(f.y).min(f.z);

        let (mut t, t_max) = {
            if t0 < 1e-3 && t1 > 1e-3 {
                (self.step, t1)
            } else if t0 >= 1e-3 && t0 < t1 {
                (t0, t1)
            } else {
                return Hit::no_hit();
            }
        };

        let sign = self.contains(ray.position(t), 0.0);
        while t < t_max {
            t += self.step;
            let v = ray.position(t);
            if self.contains(v, 0.0) != sign && self.bx.contains(v) {
                return Hit::new(t);
            }
        }
        Hit::no_hit()
    }

    fn paths(&self, args: &RenderArgs) -> Paths {
        match self.texture {
            FunctionTexture::Grid => self.paths_grid(args, 1.0 / 8.0),
            FunctionTexture::Swirl => self.paths_swirl(),
            FunctionTexture::Spiral => self.paths_spiral(),
        }
    }
}

impl<F> Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    /// Calculate max radius for radial textures based on bbox dimensions
    fn max_radius(&self) -> f64 {
        (self.bx.max.x - self.bx.min.x).max(self.bx.max.y - self.bx.min.y) / 2.0
            * std::f64::consts::SQRT_2
    }

    /// Grid texture - lines along constant x and y (works with any function)
    fn paths_grid(&self, args: &RenderArgs, grid_size: f64) -> Paths {
        let mut paths = Vec::new();
        let step_sq = args.step.powi(2);

        // Lines along constant x
        let mut x = self.bx.min.x;
        let (a, b) = (self.bx.min.y, self.bx.max.y);
        while x <= self.bx.max.x {
            let f = |y| (self.func)(x, y).min(self.bx.max.z).max(self.bx.min.z);
            let mut path = vec![Vector::new(x, a, f(a))];
            recursive_subdivide(
                ((a, path[0].z), (b, f(b))),
                &|(a, _), (b, _)| {
                    let mid = (a + b) / 2.0;
                    (mid, f(mid))
                },
                &|(a, fa), (b, fb)| {
                    let sa = args.screen_mat.mul_position_w(Vector::new(x, a, fa));
                    let sb = args.screen_mat.mul_position_w(Vector::new(x, b, fb));
                    sa.distance_squared(sb) < step_sq || (a - b).powi(2) < crate::common::EPS
                },
                &mut |(y, fy)| path.push(Vector::new(x, y, fy)),
            );
            paths.push(zvisible_offset(path, args.eye));
            x += grid_size;
        }

        // Lines along constant y
        let mut y = self.bx.min.y;
        let (a, b) = (self.bx.min.x, self.bx.max.x);
        while y <= self.bx.max.y {
            let f = |x| (self.func)(x, y).min(self.bx.max.z).max(self.bx.min.z);
            let mut path = vec![Vector::new(a, y, f(a))];
            recursive_subdivide(
                ((a, path[0].z), (b, f(b))),
                &|(a, _), (b, _)| {
                    let mid = (a + b) / 2.0;
                    (mid, f(mid))
                },
                &|(a, fa), (b, fb)| {
                    let sa = args.screen_mat.mul_position_w(Vector::new(a, y, fa));
                    let sb = args.screen_mat.mul_position_w(Vector::new(b, y, fb));
                    sa.distance_squared(sb) < step_sq || (a - b).powi(2) < crate::common::EPS
                },
                &mut |(x, fx)| path.push(Vector::new(x, y, fx)),
            );
            paths.push(zvisible_offset(path, args.eye));
            y += grid_size;
        }

        Paths::from_vec(paths)
    }

    /// Swirl texture - radial lines with twist effect (best for negative z functions)
    fn paths_swirl(&self) -> Paths {
        let mut paths = Vec::new();
        let fine = 1.0 / 256.0;
        let max_radius = self.max_radius();

        let mut a = 0;
        while a < 360 {
            let mut path = Vec::new();
            let mut r = 0.0;
            while r <= max_radius {
                let x = radians(a as f64).cos() * r;
                let y = radians(a as f64).sin() * r;
                let mut z = (self.func)(x, y);
                // Only apply swirl effect when z is negative to avoid NaN
                let o = if z < 0.0 { -(-z).powf(1.4) } else { 0.0 };
                let x = (radians(a as f64) - o).cos() * r;
                let y = (radians(a as f64) - o).sin() * r;
                z = z.min(self.bx.max.z).max(self.bx.min.z);

                // Check if point is within bbox x/y bounds
                if x >= self.bx.min.x
                    && x <= self.bx.max.x
                    && y >= self.bx.min.y
                    && y <= self.bx.max.y
                {
                    path.push(Vector::new(x, y, z));
                } else {
                    // Point is outside bbox, start a new path segment
                    if path.len() > 1 {
                        paths.push(path);
                    }
                    path = Vec::new();
                }
                r += fine;
            }
            if path.len() > 1 {
                paths.push(path);
            }
            a += 5;
        }

        Paths::from_vec(paths)
    }

    /// Spiral texture - single spiral path (works with any function)
    fn paths_spiral(&self) -> Paths {
        let mut paths = Vec::new();
        let mut path = Vec::new();
        let n = 10000;
        let max_radius = self.max_radius();

        for i in 0..n {
            let t = i as f64 / n as f64;
            let r = max_radius - t * max_radius;
            let x = radians(t * 2.0 * std::f64::consts::PI * 3000.0).cos() * r;
            let y = radians(t * 2.0 * std::f64::consts::PI * 3000.0).sin() * r;
            let mut z = (self.func)(x, y);
            z = z.min(self.bx.max.z).max(self.bx.min.z);

            // Check if point is within bbox x/y bounds
            if x >= self.bx.min.x && x <= self.bx.max.x && y >= self.bx.min.y && y <= self.bx.max.y
            {
                path.push(Vector::new(x, y, z));
            } else {
                // Point is outside bbox, start a new path segment
                if path.len() > 1 {
                    paths.push(path);
                }
                path = Vec::new();
            }
        }

        if path.len() > 1 {
            paths.push(path);
        }

        Paths::from_vec(paths)
    }
}

fn zvisible_offset(path: Vec<Vector>, eye: Vector) -> Vec<Vector> {
    let mut offsets = vec![0.0f64; path.len()];
    let ez = eye.z;
    for i in 1..path.len() - 1 {
        let (a, c, b) = (path[i - 1], path[i], path[i + 1]);
        let z = a.z
            + (b.z - a.z)
                * (((a.x - c.x).powi(2) + (a.y - c.y).powi(2))
                    / ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)))
                .sqrt();
        let offset = if (c.z > z) == (ez > z) { c.z - z } else { 0.0 };
        if offset.abs() > offsets[i - 1].abs() {
            offsets[i - 1] = offset;
        }
        if offset.abs() > offsets[i + 1].abs() {
            offsets[i + 1] = offset;
        }
    }
    path.into_iter()
        .zip(offsets)
        .map(|(mut v, z)| {
            v.z += z;
            v
        })
        .collect()
}

use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::radians;
use crate::vector::Vector;

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

pub struct Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    pub func: F,
    pub bx: Box,
    pub direction: Direction,
    pub texture: FunctionTexture,
}

impl<F> Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    pub fn new(func: F, bx: Box, direction: Direction) -> Self {
        Function { func, bx, direction, texture: FunctionTexture::default() }
    }

    pub fn with_texture(mut self, texture: FunctionTexture) -> Self {
        self.texture = texture;
        self
    }
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
        let step = 1.0 / 64.0;
        let sign = self.contains(ray.position(step), 0.0);
        
        let mut t = step;
        while t < 10.0 {
            let v = ray.position(t);
            if self.contains(v, 0.0) != sign && self.bx.contains(v) {
                return Hit::new(t);
            }
            t += step;
        }
        Hit::no_hit()
    }

    fn paths(&self) -> Paths {
        match self.texture {
            FunctionTexture::Grid => self.paths_grid(),
            FunctionTexture::Swirl => self.paths_swirl(),
            FunctionTexture::Spiral => self.paths_spiral(),
        }
    }
}

impl<F> Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    /// Grid texture - lines along constant x and y (works with any function)
    fn paths_grid(&self) -> Paths {
        let mut paths = Vec::new();
        let step = 1.0 / 8.0;
        let fine = 1.0 / 64.0;

        // Lines along constant x
        let mut x = self.bx.min.x;
        while x <= self.bx.max.x {
            let mut path = Vec::new();
            let mut y = self.bx.min.y;
            while y <= self.bx.max.y {
                let mut z = (self.func)(x, y);
                z = z.min(self.bx.max.z).max(self.bx.min.z);
                path.push(Vector::new(x, y, z));
                y += fine;
            }
            paths.push(path);
            x += step;
        }

        // Lines along constant y
        let mut y = self.bx.min.y;
        while y <= self.bx.max.y {
            let mut path = Vec::new();
            let mut x = self.bx.min.x;
            while x <= self.bx.max.x {
                let mut z = (self.func)(x, y);
                z = z.min(self.bx.max.z).max(self.bx.min.z);
                path.push(Vector::new(x, y, z));
                x += fine;
            }
            paths.push(path);
            y += step;
        }

        Paths::from_vec(paths)
    }

    /// Swirl texture - radial lines with twist effect (best for negative z functions)
    fn paths_swirl(&self) -> Paths {
        let mut paths = Vec::new();
        let fine = 1.0 / 256.0;

        let mut a = 0;
        while a < 360 {
            let mut path = Vec::new();
            let mut r = 0.0;
            while r <= 8.0 {
                let x = radians(a as f64).cos() * r;
                let y = radians(a as f64).sin() * r;
                let mut z = (self.func)(x, y);
                // Only apply swirl effect when z is negative to avoid NaN
                let o = if z < 0.0 { -(-z).powf(1.4) } else { 0.0 };
                let x = (radians(a as f64) - o).cos() * r;
                let y = (radians(a as f64) - o).sin() * r;
                z = z.min(self.bx.max.z).max(self.bx.min.z);
                path.push(Vector::new(x, y, z));
                r += fine;
            }
            paths.push(path);
            a += 5;
        }

        Paths::from_vec(paths)
    }

    /// Spiral texture - single spiral path (works with any function)
    fn paths_spiral(&self) -> Paths {
        let mut path = Vec::new();
        let n = 10000;

        for i in 0..n {
            let t = i as f64 / n as f64;
            let r = 8.0 - t.powf(0.1) * 8.0;
            let x = radians(t * 2.0 * std::f64::consts::PI * 3000.0).cos() * r;
            let y = radians(t * 2.0 * std::f64::consts::PI * 3000.0).sin() * r;
            let mut z = (self.func)(x, y);
            z = z.min(self.bx.max.z).max(self.bx.min.z);
            path.push(Vector::new(x, y, z));
        }

        Paths::from_vec(vec![path])
    }
}

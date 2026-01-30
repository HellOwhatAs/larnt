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

pub struct Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    pub func: F,
    pub bx: Box,
    pub direction: Direction,
}

impl<F> Function<F>
where
    F: Fn(f64, f64) -> f64 + Send + Sync,
{
    pub fn new(func: F, bx: Box, direction: Direction) -> Self {
        Function { func, bx, direction }
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
                let o = -(-z).powf(1.4);
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
}

use crate::vector::Vector;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector,
    pub direction: Vector,
}

impl Ray {
    pub fn new(origin: Vector, direction: Vector) -> Self {
        Ray { origin, direction }
    }

    pub fn position(&self, t: f64) -> Vector {
        self.origin.add(self.direction.mul_scalar(t))
    }
}

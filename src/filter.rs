use crate::bounding_box::BBox;
use crate::matrix::Matrix;
use crate::vector::Vector;

pub static CLIP_BOX: BBox = BBox {
    min: Vector {
        x: -1.0,
        y: -1.0,
        z: -1.0,
    },
    max: Vector {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    },
};

pub trait Filter {
    fn filter(&self, v: Vector) -> Option<Vector>;
}

pub struct ClipFilter<F> {
    pub matrix: Matrix,
    pub eye: Vector,
    pub visible: F,
}

impl<F> ClipFilter<F> {
    pub fn new(matrix: Matrix, eye: Vector, visible: F) -> Self {
        Self {
            matrix,
            eye,
            visible,
        }
    }
}

impl<F: Fn(Vector, Vector) -> bool> Filter for ClipFilter<F> {
    fn filter(&self, v: Vector) -> Option<Vector> {
        let w = self.matrix.mul_position_w(v);
        if !CLIP_BOX.contains(w) {
            return None;
        }
        if !(self.visible)(self.eye, v) {
            return None;
        }
        Some(w)
    }
}

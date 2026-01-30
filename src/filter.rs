use crate::bounding_box::Box;
use crate::matrix::Matrix;
use crate::scene::Scene;
use crate::vector::Vector;

pub trait Filter {
    fn filter(&self, v: Vector) -> Option<Vector>;
}

pub struct ClipFilter<'a> {
    pub matrix: Matrix,
    pub eye: Vector,
    pub scene: &'a Scene,
}

pub static CLIP_BOX: Box = Box {
    min: Vector { x: -1.0, y: -1.0, z: -1.0 },
    max: Vector { x: 1.0, y: 1.0, z: 1.0 },
};

impl<'a> Filter for ClipFilter<'a> {
    fn filter(&self, v: Vector) -> Option<Vector> {
        let w = self.matrix.mul_position_w(v);
        if !self.scene.visible(self.eye, v) {
            return None;
        }
        if !CLIP_BOX.contains(w) {
            return None;
        }
        Some(w)
    }
}

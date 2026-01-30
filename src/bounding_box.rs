use crate::axis::Axis;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::triangle::Triangle;
use crate::vector::Vector;

#[derive(Debug, Clone, Copy, Default)]
pub struct Box {
    pub min: Vector,
    pub max: Vector,
}

impl Box {
    pub fn new(min: Vector, max: Vector) -> Self {
        Box { min, max }
    }

    pub fn for_shapes(shapes: &[&dyn Shape]) -> Self {
        if shapes.is_empty() {
            return Box::default();
        }
        let mut bx = shapes[0].bounding_box();
        for shape in shapes.iter().skip(1) {
            bx = bx.extend(shape.bounding_box());
        }
        bx
    }

    pub fn for_triangles(triangles: &[Triangle]) -> Self {
        if triangles.is_empty() {
            return Box::default();
        }
        let mut bx = triangles[0].bounding_box();
        for tri in triangles.iter().skip(1) {
            bx = bx.extend(tri.bounding_box());
        }
        bx
    }

    pub fn for_vectors(vectors: &[Vector]) -> Self {
        if vectors.is_empty() {
            return Box::default();
        }
        let mut min = vectors[0];
        let mut max = vectors[0];
        for v in vectors.iter().skip(1) {
            min = min.min(*v);
            max = max.max(*v);
        }
        Box { min, max }
    }

    pub fn anchor(&self, anchor: Vector) -> Vector {
        self.min.add(self.size().mul(anchor))
    }

    pub fn center(&self) -> Vector {
        self.anchor(Vector::new(0.5, 0.5, 0.5))
    }

    pub fn size(&self) -> Vector {
        self.max.sub(self.min)
    }

    pub fn contains(&self, v: Vector) -> bool {
        self.min.x <= v.x && self.max.x >= v.x &&
        self.min.y <= v.y && self.max.y >= v.y &&
        self.min.z <= v.z && self.max.z >= v.z
    }

    pub fn extend(&self, other: Box) -> Box {
        Box {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn intersect(&self, r: Ray) -> (f64, f64) {
        let x1 = (self.min.x - r.origin.x) / r.direction.x;
        let y1 = (self.min.y - r.origin.y) / r.direction.y;
        let z1 = (self.min.z - r.origin.z) / r.direction.z;
        let x2 = (self.max.x - r.origin.x) / r.direction.x;
        let y2 = (self.max.y - r.origin.y) / r.direction.y;
        let z2 = (self.max.z - r.origin.z) / r.direction.z;

        let (x1, x2) = if x1 > x2 { (x2, x1) } else { (x1, x2) };
        let (y1, y2) = if y1 > y2 { (y2, y1) } else { (y1, y2) };
        let (z1, z2) = if z1 > z2 { (z2, z1) } else { (z1, z2) };

        let t1 = x1.max(y1).max(z1);
        let t2 = x2.min(y2).min(z2);
        (t1, t2)
    }

    pub fn partition(&self, axis: Axis, point: f64) -> (bool, bool) {
        match axis {
            Axis::X => (self.min.x <= point, self.max.x >= point),
            Axis::Y => (self.min.y <= point, self.max.y >= point),
            Axis::Z => (self.min.z <= point, self.max.z >= point),
            Axis::None => (false, false),
        }
    }
}

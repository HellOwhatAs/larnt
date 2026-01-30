use crate::axis::Axis;
use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::util::median;
use std::sync::Arc;

pub struct Tree {
    pub bx: Box,
    pub root: Node,
}

impl Tree {
    pub fn new(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Self {
        let bx = box_for_arc_shapes(&shapes);
        let mut root = Node::new(shapes);
        root.split(0);
        Tree { bx, root }
    }

    pub fn intersect(&self, r: Ray) -> Hit {
        let (tmin, tmax) = self.bx.intersect(r);
        if tmax < tmin || tmax <= 0.0 {
            return Hit::no_hit();
        }
        self.root.intersect(r, tmin, tmax)
    }
}

fn box_for_arc_shapes(shapes: &[Arc<dyn Shape + Send + Sync>]) -> Box {
    if shapes.is_empty() {
        return Box::default();
    }
    let mut bx = shapes[0].bounding_box();
    for shape in shapes.iter().skip(1) {
        bx = bx.extend(shape.bounding_box());
    }
    bx
}

pub struct Node {
    pub axis: Axis,
    pub point: f64,
    pub shapes: Vec<Arc<dyn Shape + Send + Sync>>,
    pub left: Option<std::boxed::Box<Node>>,
    pub right: Option<std::boxed::Box<Node>>,
}

impl Node {
    pub fn new(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Self {
        Node {
            axis: Axis::None,
            point: 0.0,
            shapes,
            left: None,
            right: None,
        }
    }

    pub fn intersect(&self, r: Ray, tmin: f64, tmax: f64) -> Hit {
        let (tsplit, left_first) = match self.axis {
            Axis::None => return self.intersect_shapes(r),
            Axis::X => {
                let tsplit = (self.point - r.origin.x) / r.direction.x;
                let left_first = r.origin.x < self.point || (r.origin.x == self.point && r.direction.x <= 0.0);
                (tsplit, left_first)
            }
            Axis::Y => {
                let tsplit = (self.point - r.origin.y) / r.direction.y;
                let left_first = r.origin.y < self.point || (r.origin.y == self.point && r.direction.y <= 0.0);
                (tsplit, left_first)
            }
            Axis::Z => {
                let tsplit = (self.point - r.origin.z) / r.direction.z;
                let left_first = r.origin.z < self.point || (r.origin.z == self.point && r.direction.z <= 0.0);
                (tsplit, left_first)
            }
        };

        // SAFETY: left and right children always exist when axis != Axis::None
        // This invariant is maintained by the split() method
        let (first, second) = if left_first {
            (
                self.left.as_ref().expect("left child must exist when axis is set"),
                self.right.as_ref().expect("right child must exist when axis is set")
            )
        } else {
            (
                self.right.as_ref().expect("right child must exist when axis is set"),
                self.left.as_ref().expect("left child must exist when axis is set")
            )
        };

        if tsplit > tmax || tsplit <= 0.0 {
            first.intersect(r, tmin, tmax)
        } else if tsplit < tmin {
            second.intersect(r, tmin, tmax)
        } else {
            let h1 = first.intersect(r, tmin, tsplit);
            if h1.t <= tsplit {
                h1
            } else {
                let h2 = second.intersect(r, tsplit, tmax.min(h1.t));
                if h1.t <= h2.t { h1 } else { h2 }
            }
        }
    }

    fn intersect_shapes(&self, r: Ray) -> Hit {
        let mut hit = Hit::no_hit();
        for shape in &self.shapes {
            let h = shape.intersect(r);
            if h.t < hit.t {
                hit = h;
            }
        }
        hit
    }

    fn partition_score(&self, axis: Axis, point: f64) -> usize {
        let mut left = 0;
        let mut right = 0;
        for shape in &self.shapes {
            let bx = shape.bounding_box();
            let (l, r) = bx.partition(axis, point);
            if l { left += 1; }
            if r { right += 1; }
        }
        left.max(right)
    }

    fn partition(&self, axis: Axis, point: f64) -> (Vec<Arc<dyn Shape + Send + Sync>>, Vec<Arc<dyn Shape + Send + Sync>>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for shape in &self.shapes {
            let bx = shape.bounding_box();
            let (l, r) = bx.partition(axis, point);
            if l { left.push(Arc::clone(shape)); }
            if r { right.push(Arc::clone(shape)); }
        }
        (left, right)
    }

    pub fn split(&mut self, depth: usize) {
        if self.shapes.len() < 8 {
            return;
        }

        let mut xs = Vec::with_capacity(self.shapes.len() * 2);
        let mut ys = Vec::with_capacity(self.shapes.len() * 2);
        let mut zs = Vec::with_capacity(self.shapes.len() * 2);

        for shape in &self.shapes {
            let bx = shape.bounding_box();
            xs.push(bx.min.x);
            xs.push(bx.max.x);
            ys.push(bx.min.y);
            ys.push(bx.max.y);
            zs.push(bx.min.z);
            zs.push(bx.max.z);
        }

        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        zs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mx = median(&xs);
        let my = median(&ys);
        let mz = median(&zs);

        let mut best = (self.shapes.len() as f64 * 0.85) as usize;
        let mut best_axis = Axis::None;
        let mut best_point = 0.0;

        let sx = self.partition_score(Axis::X, mx);
        if sx < best {
            best = sx;
            best_axis = Axis::X;
            best_point = mx;
        }

        let sy = self.partition_score(Axis::Y, my);
        if sy < best {
            best = sy;
            best_axis = Axis::Y;
            best_point = my;
        }

        let sz = self.partition_score(Axis::Z, mz);
        if sz < best {
            best_axis = Axis::Z;
            best_point = mz;
        }

        if best_axis == Axis::None {
            return;
        }

        let (l, r) = self.partition(best_axis, best_point);
        self.axis = best_axis;
        self.point = best_point;

        let mut left = Node::new(l);
        let mut right = Node::new(r);
        left.split(depth + 1);
        right.split(depth + 1);

        self.left = Some(std::boxed::Box::new(left));
        self.right = Some(std::boxed::Box::new(right));
        self.shapes.clear();
    }
}

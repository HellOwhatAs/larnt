use crate::axis::Axis;
use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::vector::Vector;
use std::sync::Arc;

#[derive(Clone)]
struct BvhNode {
    pub bx: Box,
    pub left_first: usize,
    pub count: usize,
    pub axis: Axis,
}

impl BvhNode {
    fn empty() -> Self {
        Self {
            bx: Box::default(),
            left_first: 0,
            count: 0,
            axis: Axis::None,
        }
    }
}

struct PrimInfo {
    shape: Arc<dyn Shape + Send + Sync>,
    bx: Box,
    centroid: (f64, f64, f64),
}

pub struct Tree {
    nodes: Vec<BvhNode>,
    shapes: Vec<Arc<dyn Shape + Send + Sync>>,
}

impl Tree {
    pub fn new(shapes: Vec<Arc<dyn Shape + Send + Sync>>) -> Self {
        if shapes.is_empty() {
            return Tree {
                nodes: Vec::new(),
                shapes,
            };
        }

        let len = shapes.len();
        let mut nodes = Vec::with_capacity(len * 2);
        nodes.push(BvhNode::empty());

        let mut prims: Vec<PrimInfo> = shapes
            .into_iter()
            .map(|shape| {
                let bx = shape.bounding_box();
                let centroid = Self::centroid(&bx);
                PrimInfo {
                    shape,
                    bx,
                    centroid,
                }
            })
            .collect();

        let mut sah_right_boxes = vec![Box::default(); len];

        Self::build(&mut nodes, &mut prims, &mut sah_right_boxes, 0, 0, len);

        let sorted_shapes = prims.into_iter().map(|p| p.shape).collect();

        Tree {
            nodes,
            shapes: sorted_shapes,
        }
    }

    pub fn intersect(&self, r: Ray) -> Hit {
        if self.nodes.is_empty() {
            return Hit::no_hit();
        }

        let mut closest_hit = Hit::no_hit();
        let mut stack = [0usize; 64];
        let mut stack_ptr = 1;
        stack[0] = 0;

        let Vector {
            x: dir_x,
            y: dir_y,
            z: dir_z,
        } = r.direction;

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_idx = stack[stack_ptr];
            let node = &self.nodes[node_idx];

            let (tmin, tmax) = node.bx.intersect(r);

            if tmax < tmin || tmax <= 0.0 || tmin >= closest_hit.t {
                continue;
            }

            let is_dir_negative = match (node.axis, node.count > 0) {
                (Axis::None, _) | (_, true) => {
                    for i in 0..node.count {
                        let shape = &self.shapes[node.left_first + i];
                        let hit = shape.intersect(r);
                        if hit.t < closest_hit.t {
                            closest_hit = hit;
                        }
                    }
                    continue;
                }
                (Axis::X, _) => dir_x < 0.0,
                (Axis::Y, _) => dir_y < 0.0,
                (Axis::Z, _) => dir_z < 0.0,
            };

            let (near, far) = if is_dir_negative {
                (node.left_first + 1, node.left_first)
            } else {
                (node.left_first, node.left_first + 1)
            };

            stack[stack_ptr] = far;
            stack_ptr += 1;
            stack[stack_ptr] = near;
            stack_ptr += 1;
        }

        closest_hit
    }

    fn build(
        nodes: &mut Vec<BvhNode>,
        prims: &mut [PrimInfo],
        sah_right_boxes: &mut [Box],
        node_idx: usize,
        start: usize,
        end: usize,
    ) {
        let count = end - start;

        let mut parent_bx = prims[start].bx;
        for i in start + 1..end {
            parent_bx = parent_bx.extend(prims[i].bx);
        }
        nodes[node_idx].bx = parent_bx;

        if count <= 2 {
            nodes[node_idx].left_first = start;
            nodes[node_idx].count = count;
            nodes[node_idx].axis = Axis::None;
            return;
        }

        let mut min_c = prims[start].centroid;
        let mut max_c = min_c;
        for i in start + 1..end {
            let c = prims[i].centroid;
            min_c.0 = min_c.0.min(c.0);
            min_c.1 = min_c.1.min(c.1);
            min_c.2 = min_c.2.min(c.2);
            max_c.0 = max_c.0.max(c.0);
            max_c.1 = max_c.1.max(c.1);
            max_c.2 = max_c.2.max(c.2);
        }

        let extent_x = max_c.0 - min_c.0;
        let extent_y = max_c.1 - min_c.1;
        let extent_z = max_c.2 - min_c.2;

        let axis = if extent_x > extent_y && extent_x > extent_z {
            Axis::X
        } else if extent_y > extent_z {
            Axis::Y
        } else {
            Axis::Z
        };
        nodes[node_idx].axis = axis;

        prims[start..end].sort_unstable_by(|a, b| {
            let [va, vb] = match axis {
                Axis::X => [a, b].map(|x| x.centroid.0),
                Axis::Y => [a, b].map(|x| x.centroid.1),
                Axis::Z => [a, b].map(|x| x.centroid.2),
                _ => unreachable!("The axis should never be None here"),
            };
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut current_right_bx = prims[end - 1].bx;
        sah_right_boxes[count - 1] = current_right_bx;
        for i in (1..count - 1).rev() {
            current_right_bx = current_right_bx.extend(prims[start + i].bx);
            sah_right_boxes[i] = current_right_bx;
        }

        let parent_area = Self::surface_area(&parent_bx);
        let mut current_left_bx = prims[start].bx;

        let mut best_cost = f64::MAX;
        let mut best_split_idx = start + count / 2;

        for i in 1..count {
            let left_count = i;
            let right_count = count - i;

            let left_area = Self::surface_area(&current_left_bx);
            let right_area = Self::surface_area(&sah_right_boxes[i]);

            let cost = left_area * (left_count as f64) + right_area * (right_count as f64);

            if cost < best_cost {
                best_cost = cost;
                best_split_idx = start + i;
            }

            current_left_bx = current_left_bx.extend(prims[start + i].bx);
        }

        let traversal_cost = parent_area * 0.125;
        let leaf_cost = (count as f64) * parent_area;

        if best_cost + traversal_cost >= leaf_cost && count <= 8 {
            nodes[node_idx].left_first = start;
            nodes[node_idx].count = count;
            return;
        }

        let left_child_idx = nodes.len();
        nodes.push(BvhNode::empty());
        nodes.push(BvhNode::empty());

        nodes[node_idx].left_first = left_child_idx;
        nodes[node_idx].count = 0;

        Self::build(
            nodes,
            prims,
            sah_right_boxes,
            left_child_idx,
            start,
            best_split_idx,
        );
        Self::build(
            nodes,
            prims,
            sah_right_boxes,
            left_child_idx + 1,
            best_split_idx,
            end,
        );
    }

    fn centroid(bx: &Box) -> (f64, f64, f64) {
        (
            (bx.min.x + bx.max.x) * 0.5,
            (bx.min.y + bx.max.y) * 0.5,
            (bx.min.z + bx.max.z) * 0.5,
        )
    }

    fn surface_area(bx: &Box) -> f64 {
        let dx = (bx.max.x - bx.min.x).max(0.0);
        let dy = (bx.max.y - bx.min.y).max(0.0);
        let dz = (bx.max.z - bx.min.z).max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }
}

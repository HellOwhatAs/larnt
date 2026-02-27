use crate::bounding_box::BBox;
use crate::common::EPS;
use crate::hit::Hit;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::tree::Tree;
use crate::triangle::Triangle;
use crate::vector::Vector;
use crate::{Matrix, TransformedShape};
use std::collections::{HashMap, HashSet};

/// Triangle mesh shape.
pub struct Mesh {
    bx: BBox,
    pub vertices: Vec<Vector>,
    pub triangles: Vec<usize>,
    tree: Tree<Triangle>,
    pub flipped_triangles: Option<HashSet<(usize, usize)>>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vector>, triangles: Vec<usize>) -> Self {
        let tree = Tree::new(
            triangles
                .chunks_exact(3)
                .map(|w| Triangle::new(vertices[w[0]], vertices[w[1]], vertices[w[2]]))
                .collect(),
        );
        Self {
            bx: BBox::for_vectors(&vertices),
            triangles,
            vertices,
            tree,
            flipped_triangles: None,
        }
    }

    pub fn from_triangles(triangles: Vec<Triangle>) -> Self {
        let mut merger = VertexMerger::new(1e-6);
        let itriangles = triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3].map(|v| merger.get_or_insert(v)))
            .collect();
        Self {
            bx: BBox::for_shapes(&triangles),
            triangles: itriangles,
            vertices: merger.vertices,
            tree: Tree::new(triangles),
            flipped_triangles: None,
        }
    }

    pub fn fit_inside(&self, bx: BBox, anchor: Vector) -> Matrix {
        let scale = bx.size().div(self.bx.size()).min_component();
        let extra = bx.size().sub(self.bx.size().mul_scalar(scale));
        let mut matrix = Matrix::identity();
        matrix = matrix.translated(self.bx.min.mul_scalar(-1.0));
        matrix = matrix.scaled(Vector::new(scale, scale, scale));
        matrix = matrix.translated(bx.min.add(extra.mul(anchor)));
        matrix
    }

    pub fn parametric_surface(
        points: Vec<Vector>,
        u_steps: usize,
        v_steps: usize,
        indexer: impl Fn(usize, usize) -> usize,
    ) -> Self {
        let (u_mapper, v_mapper) = {
            let upoints =
                |u| -> Vec<Vector> { (0..v_steps).map(|v| points[indexer(u, v)]).collect() };
            let vpoints =
                |v| -> Vec<Vector> { (0..u_steps).map(|u| points[indexer(u, v)]).collect() };
            let [u0, ue] = [0, u_steps].map(upoints);
            let [v0, ve] = [0, v_steps].map(vpoints);
            let u_mapper = CyclicMapping::check_equal(&u0, &ue);
            let v_mapper = CyclicMapping::check_equal(&v0, &ve);
            (u_mapper, v_mapper)
        };

        let mut triangles = Vec::with_capacity(u_steps * v_steps * 6);
        let mut build_triangles = |u: usize, v: usize| -> (Option<usize>, Option<usize>) {
            let u_mapper = u_mapper.filter(|_| u == u_steps - 1);
            let v_mapper = v_mapper.filter(|_| v == v_steps - 1);

            let get_idx = |du: bool, dv: bool| {
                let base_u = if du { u + 1 } else { u };
                let base_v = if dv { v + 1 } else { v };
                let pu = match (du, u_mapper, dv, v_mapper) {
                    (true, Some(_), _, _) => 0,
                    (_, _, true, Some(vmap)) => vmap.map_index_inv(base_u),
                    _ => base_u,
                };
                let pv = match (dv, v_mapper, du, u_mapper) {
                    (true, Some(_), _, _) => 0,
                    (_, _, true, Some(umap)) => umap.map_index_inv(base_v),
                    _ => base_v,
                };
                indexer(pu, pv)
            };

            let i00 = get_idx(false, false);
            let i10 = get_idx(true, false);
            let i01 = get_idx(false, true);
            let i11 = get_idx(true, true);

            let mut add_triangle = |a, b, c| {
                (a != b && a != c && b != c).then(|| {
                    let tri_idx = triangles.len() / 3;
                    triangles.extend([a, b, c]);
                    tri_idx
                })
            };
            let prev = add_triangle(i00, i10, i01);
            let next = add_triangle(i10, i11, i01);
            (prev, next)
        };

        let flipped_triangles = {
            let mut flipped_triangles: Option<HashSet<(usize, usize)>> = None;
            let mut u_rev = u_mapper
                .is_some_and(|m| m.is_reverse())
                .then(|| vec![None; v_steps * 2]);
            let mut v_rev = v_mapper
                .is_some_and(|m| m.is_reverse())
                .then(|| vec![None; u_steps * 2]);

            for u in 0..u_steps {
                for v in 0..v_steps {
                    let (prev_tri, next_tri) = build_triangles(u, v);
                    if let Some(uarr) = &mut u_rev {
                        if u == 0 {
                            uarr[v] = prev_tri;
                        }
                        if u == u_steps - 1 {
                            uarr[v_steps + v] = next_tri;
                        }
                    }
                    if let Some(varr) = &mut v_rev {
                        if v == 0 {
                            varr[u] = prev_tri;
                        }
                        if v == v_steps - 1 {
                            varr[u_steps + u] = next_tri;
                        }
                    }
                }
            }

            let mut process_flipped =
                |x0: &[Option<usize>], xe: &[Option<usize>], ysteps: usize, xmap: CyclicMapping| {
                    for (ib, b) in xe
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(i, ob)| ob.map(|b| (i, b)))
                    {
                        let ib_next = (ib + 1) % ysteps;
                        let ia = xmap.map_index_inv(ib_next);
                        if let Some(a) = x0[ia] {
                            let flipped = flipped_triangles.get_or_insert_default();
                            flipped.insert(if a < b { (a, b) } else { (b, a) });
                        }
                    }
                };

            if let (Some(uarr), Some(umap)) = (u_rev, u_mapper) {
                process_flipped(&uarr[..v_steps], &uarr[v_steps..], v_steps, umap);
            }
            if let (Some(varr), Some(vmap)) = (v_rev, v_mapper) {
                process_flipped(&varr[..u_steps], &varr[u_steps..], u_steps, vmap);
            }
            flipped_triangles
        };

        let mut result = Self::new(points, triangles);
        result.flipped_triangles = flipped_triangles;
        result
    }

    pub fn filter_paths(&self, group_keeper: impl Fn(&[(usize, usize, usize)]) -> bool) -> Paths {
        let mut paths = Paths::new();
        if self.triangles.len() < 3 {
            return paths;
        }

        let edges = {
            let mut edges = self
                .triangles
                .chunks_exact(3)
                .enumerate()
                .flat_map(|(face_idx, chunk)| {
                    let [i1, i2, i3] = [chunk[0], chunk[1], chunk[2]];
                    [
                        (i1.min(i2), i1.max(i2), face_idx),
                        (i2.min(i3), i2.max(i3), face_idx),
                        (i3.min(i1), i3.max(i1), face_idx),
                    ]
                })
                .collect::<Vec<_>>();
            edges.sort_unstable_by_key(|e| (e.0, e.1));
            edges
        };

        let mut i = 0;
        while i < edges.len() {
            let current_edge = (edges[i].0, edges[i].1);
            let mut count = 1;

            while i + count < edges.len()
                && (edges[i + count].0, edges[i + count].1) == current_edge
            {
                count += 1;
            }

            if group_keeper(&edges[i..i + count]) {
                paths
                    .new_path()
                    .extend([self.vertices[current_edge.0], self.vertices[current_edge.1]]);
            }

            i += count;
        }

        paths
    }

    pub fn vanilla_paths(&self, _args: &RenderArgs) -> Paths {
        let face_normals: Vec<Vector> = self
            .triangles
            .chunks_exact(3)
            .map(|chunk| normalized_normal(chunk.iter().map(|&i| self.vertices[i])))
            .collect();
        self.filter_paths(|edges| {
            if edges.len() == 1 {
                true
            } else {
                let base_normal = face_normals[edges[0].2];
                edges.iter().skip(1).any(|e| {
                    let other_normal = face_normals[e.2];
                    base_normal.distance_squared(other_normal) > crate::common::EPS
                })
            }
        })
    }

    pub fn silhouette_paths(&self, args: &RenderArgs) -> Paths {
        let face_data: Vec<_> = self
            .triangles
            .chunks_exact(3)
            .map(|chunk| {
                let [v1, v2, v3] = [0, 1, 2].map(|i| self.vertices[chunk[i]]);
                let true_normal = (v2.sub(v1)).cross(v3.sub(v1)).normalize();
                let view_dir = args.eye.sub(v1);
                true_normal.dot(view_dir)
            })
            .collect();
        self.filter_paths(|edges| {
            if edges.len() == 2 {
                let [dot1, dot2] = [0, 1].map(|i| face_data[edges[i].2]);
                if let Some(flipped) = &self.flipped_triangles
                    && flipped.contains(&{
                        let (a, b) = (edges[0].2, edges[1].2);
                        if a < b { (a, b) } else { (b, a) }
                    })
                {
                    return dot1 * dot2 > 0.0;
                }
                dot1 * dot2 <= 0.0
            } else {
                true
            }
        })
    }
}

impl AsRef<Triangle> for Triangle {
    fn as_ref(&self) -> &Triangle {
        self
    }
}

pub trait TriangleMesh {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator;
}

impl TriangleMesh for Mesh {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator {
        self.tree.shapes().iter()
    }
}

impl TriangleMesh for TransformedShape<Mesh> {
    fn triangles(&self) -> impl Iterator<Item = impl AsRef<Triangle>> + ExactSizeIterator {
        self.shape.tree.shapes().iter().map(|triangle| {
            Triangle::new(
                self.matrix.mul_position(triangle.v1),
                self.matrix.mul_position(triangle.v2),
                self.matrix.mul_position(triangle.v3),
            )
        })
    }
}

impl Shape for Mesh {
    fn bounding_box(&self) -> BBox {
        self.bx
    }

    fn contains(&self, _v: Vector, _f: f64) -> bool {
        false
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.tree.intersect(r)
    }

    fn paths(&self, args: &RenderArgs) -> Paths {
        self.silhouette_paths(args)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CyclicMapping {
    Forward(usize, usize),
    Reverse(usize, usize),
}

impl CyclicMapping {
    #[inline]
    pub fn is_reverse(&self) -> bool {
        matches!(self, CyclicMapping::Reverse(_, _))
    }

    #[inline]
    pub fn map_index(&self, a_idx: usize) -> usize {
        match *self {
            CyclicMapping::Forward(offset, len) => (offset + a_idx) % len,
            CyclicMapping::Reverse(offset, len) => (offset + len - (a_idx % len)) % len,
        }
    }

    #[inline]
    pub fn map_index_inv(&self, b_idx: usize) -> usize {
        match *self {
            CyclicMapping::Forward(offset, len) => (b_idx + len - (offset % len)) % len,
            CyclicMapping::Reverse(offset, len) => (offset + len - (b_idx % len)) % len,
        }
    }

    pub fn check_equal(a: &[Vector], b: &[Vector]) -> Option<CyclicMapping> {
        let n = a.len();
        if a.len() != b.len() || n == 0 {
            return None;
        }
        for i in 0..n {
            if a[0].distance_squared(b[i]) <= EPS {
                let (b1, b2) = b.split_at(i);
                let a_tail = &a[1..];
                let forward_match = a_tail
                    .iter()
                    .zip(b2[1..].iter().chain(b1.iter()))
                    .all(|(va, &vb)| va.distance_squared(vb) <= EPS);
                if forward_match {
                    return Some(CyclicMapping::Forward(i, n));
                }

                let reverse_match = a_tail
                    .iter()
                    .rev()
                    .zip(b2[1..].iter().chain(b1.iter()))
                    .all(|(va, &vb)| va.distance_squared(vb) <= EPS);

                if reverse_match {
                    return Some(CyclicMapping::Reverse(i, n));
                }
            }
        }
        None
    }
}

fn normalized_normal(mut v123: impl Iterator<Item = Vector>) -> Vector {
    let [v1, v2, v3] = std::array::from_fn(|_| v123.next().unwrap());
    let normal = (v2.sub(v1)).cross(v3.sub(v1)).normalize();
    if normal.x < 0.0
        || (normal.x == 0.0 && normal.y < 0.0)
        || (normal.x == 0.0 && normal.y == 0.0 && normal.z < 0.0)
    {
        normal.mul_scalar(-1.0)
    } else {
        normal
    }
}

struct VertexMerger {
    pub vertices: Vec<Vector>,
    grid: HashMap<(i64, i64, i64), Vec<usize>>,
    epsilon: f64,
    epsilon_sq: f64,
}

impl VertexMerger {
    pub fn new(epsilon: f64) -> Self {
        Self {
            vertices: Vec::new(),
            grid: HashMap::new(),
            epsilon,
            epsilon_sq: epsilon * epsilon,
        }
    }

    /// Returns the index of the existing vertex if it's close enough,
    /// or inserts a new vertex and returns its index.
    pub fn get_or_insert(&mut self, v: Vector) -> usize {
        let cell_size = self.epsilon;

        let ix = (v.x / cell_size).floor() as i64;
        let iy = (v.y / cell_size).floor() as i64;
        let iz = (v.z / cell_size).floor() as i64;

        let dxyz = (-1..=1)
            .flat_map(|dx| (-1..=1).flat_map(move |dy| (-1..=1).map(move |dz| (dx, dy, dz))));
        for (dx, dy, dz) in dxyz {
            let key = (ix + dx, iy + dy, iz + dz);

            if let Some(indices) = self.grid.get(&key) {
                for &idx in indices {
                    let existing_v = self.vertices[idx];

                    if v.distance_squared(existing_v) < self.epsilon_sq {
                        return idx;
                    }
                }
            }
        }

        let new_idx = self.vertices.len();
        self.vertices.push(v);

        self.grid.entry((ix, iy, iz)).or_default().push(new_idx);

        new_idx
    }
}

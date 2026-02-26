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
        let mut triangles = Vec::with_capacity(u_steps * v_steps * 6);
        let mut flipped_triangles = None;

        let [u0, ue] = [0, u_steps].map(|u| {
            (0..v_steps)
                .map(|v| points[indexer(u, v)])
                .collect::<Vec<_>>()
        });
        let [v0, ve] = [0, v_steps].map(|v| {
            (0..u_steps)
                .map(|u| points[indexer(u, v)])
                .collect::<Vec<_>>()
        });
        let u_mapper = CyclicMapping::check_equal(&u0, &ue);
        let v_mapper = CyclicMapping::check_equal(&v0, &ve);

        let mut build_triangles = |u: usize, v: usize| -> (usize, usize) {
            let u_mapper = u_mapper.filter(|_| u == u_steps - 1);
            let v_mapper = v_mapper.filter(|_| v == v_steps - 1);
            let (p00, p10, p01, p11) = match (u_mapper, v_mapper) {
                (None, None) => ((u, v), (u + 1, v), (u, v + 1), (u + 1, v + 1)),
                (None, Some(vmap)) => (
                    (u, v),
                    (u + 1, v),
                    (vmap.map_index_inv(u), 0),
                    (vmap.map_index_inv(u + 1), 0),
                ),
                (Some(umap), None) => (
                    (u, v),
                    (0, umap.map_index_inv(v)),
                    (u, v + 1),
                    (0, umap.map_index_inv(v + 1)),
                ),
                (Some(umap), Some(vmap)) => (
                    (u, v),
                    (0, umap.map_index_inv(v)),
                    (vmap.map_index_inv(u), 0),
                    (0, 0),
                ),
            };

            let i00 = indexer(p00.0, p00.1);
            let i10 = indexer(p10.0, p10.1);
            let i01 = indexer(p01.0, p01.1);
            let i11 = indexer(p11.0, p11.1);

            let prev = triangles.len() / 3;
            for [a, b, c] in [[i00, i10, i01], [i10, i11, i01]] {
                if a != b && a != c && b != c {
                    triangles.extend([a, b, c]);
                }
            }
            let next = triangles.len() / 3 - 1;
            (prev, next)
        };

        match (u_mapper, v_mapper) {
            (u_mapper, Some(vmap))
                if vmap.is_reverse() && u_mapper.map_or(true, |umap| !umap.is_reverse()) =>
            {
                let mut flipped = HashSet::new();
                let mut v0 = vec![usize::MAX; u_steps];
                let mut ve = vec![usize::MAX; u_steps];
                for u in 0..u_steps {
                    v0[u] = build_triangles(u, 0).0;
                }
                for v in 1..(v_steps - 1) {
                    for u in 0..u_steps {
                        build_triangles(u, v);
                    }
                }
                for u in 0..u_steps {
                    ve[u] = build_triangles(u, v_steps - 1).1;
                }
                for (ib, b) in ve.into_iter().enumerate() {
                    let ib = (ib + 1) % u_steps;
                    let ia = vmap.map_index_inv(ib);
                    let a = v0[ia];
                    flipped.insert(if a < b { (a, b) } else { (b, a) });
                }
                if !flipped.is_empty() {
                    flipped_triangles = Some(flipped);
                };
            }
            (Some(umap), v_mapper)
                if umap.is_reverse() && v_mapper.map_or(true, |vmap| !vmap.is_reverse()) =>
            {
                let mut flipped = HashSet::new();
                let mut u0 = vec![usize::MAX; v_steps];
                let mut ue = vec![usize::MAX; v_steps];
                for v in 0..v_steps {
                    u0[v] = build_triangles(0, v).0;
                }
                for u in 1..(u_steps - 1) {
                    for v in 0..v_steps {
                        build_triangles(u, v);
                    }
                }
                for v in 0..v_steps {
                    ue[v] = build_triangles(u_steps - 1, v).1;
                }
                for (ib, b) in ue.into_iter().enumerate() {
                    let ib = (ib + 1) % v_steps;
                    let ia = umap.map_index_inv(ib);
                    let a = u0[ia];
                    flipped.insert(if a < b { (a, b) } else { (b, a) });
                }
                if !flipped.is_empty() {
                    flipped_triangles = Some(flipped);
                };
            }
            (Some(umap), Some(vmap)) if umap.is_reverse() && vmap.is_reverse() => {
                let mut flipped = HashSet::new();
                let mut u0 = vec![usize::MAX; v_steps];
                let mut ue = vec![usize::MAX; v_steps];
                let mut v0 = vec![usize::MAX; u_steps];
                let mut ve = vec![usize::MAX; u_steps];
                for u in 0..u_steps {
                    v0[u] = build_triangles(u, 0).0;
                }
                u0[0] = v0[0];
                for v in 1..v_steps {
                    u0[v] = build_triangles(0, v).0;
                }
                for u in 1..(u_steps - 1) {
                    for v in 1..(v_steps - 1) {
                        build_triangles(u, v);
                    }
                }
                ue[0] = v0[u_steps - 1];
                ve[0] = u0[v_steps - 1];
                for u in 1..u_steps {
                    ve[u] = build_triangles(u, v_steps - 1).1;
                }
                for v in 1..(v_steps - 1) {
                    ue[v] = build_triangles(u_steps - 1, v).1;
                }
                ue[v_steps - 1] = ve[u_steps - 1];

                if umap.is_reverse() {
                    for (ib, b) in ue.into_iter().enumerate() {
                        let ib = (ib + 1) % v_steps;
                        let ia = umap.map_index_inv(ib);
                        let a = u0[ia];
                        flipped.insert(if a < b { (a, b) } else { (b, a) });
                    }
                }

                if vmap.is_reverse() {
                    for (ib, b) in ve.into_iter().enumerate() {
                        let ib = (ib + 1) % u_steps;
                        let ia = vmap.map_index_inv(ib);
                        let a = v0[ia];
                        flipped.insert(if a < b { (a, b) } else { (b, a) });
                    }
                }

                if !flipped.is_empty() {
                    flipped_triangles = Some(flipped);
                };
            }
            _ => {
                for u in 0..u_steps {
                    for v in 0..v_steps {
                        build_triangles(u, v);
                    }
                }
            }
        }
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

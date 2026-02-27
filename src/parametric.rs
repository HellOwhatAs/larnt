use crate::bounding_box::BBox;
use crate::common::EPS;
use crate::hit::Hit;
use crate::mesh::Mesh;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::vector::Vector;
use std::collections::HashSet;

pub struct ParametricSurface {
    mesh: Mesh,
    paths: Paths,
}

impl ParametricSurface {
    pub fn new_mesh<F>(
        func: F,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_steps: usize,
        v_steps: usize,
    ) -> Mesh
    where
        F: Fn(f64, f64) -> Vector,
    {
        let (grid, indexer) = Self::calc_grid(func, u_range, v_range, u_steps, v_steps);
        Self::mesh_from_grid(grid, u_steps, v_steps, indexer)
    }

    pub fn new<F>(
        func: F,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_steps: usize,
        v_steps: usize,
    ) -> Self
    where
        F: Fn(f64, f64) -> Vector,
    {
        let (grid, indexer) = Self::calc_grid(func, u_range, v_range, u_steps, v_steps);
        Self::from_grid(grid, u_steps, v_steps, indexer)
    }

    pub fn calc_grid<F>(
        func: F,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_steps: usize,
        v_steps: usize,
    ) -> (Vec<Vector>, impl Fn(usize, usize) -> usize)
    where
        F: Fn(f64, f64) -> Vector,
    {
        let du = (u_range.1 - u_range.0) / u_steps as f64;
        let dv = (v_range.1 - v_range.0) / v_steps as f64;

        let mut grid = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
        for i in 0..=u_steps {
            let u = u_range.0 + i as f64 * du;
            for j in 0..=v_steps {
                let v = v_range.0 + j as f64 * dv;
                grid.push(func(u, v));
            }
        }
        (grid, move |i, j| i * (v_steps + 1) + j)
    }

    pub fn from_grid(
        grid: Vec<Vector>,
        u_steps: usize,
        v_steps: usize,
        indexer: impl Fn(usize, usize) -> usize,
    ) -> Self {
        Self {
            paths: Self::grid_paths(|u, v| grid[indexer(u, v)], u_steps, v_steps),
            mesh: Self::mesh_from_grid(grid, u_steps, v_steps, indexer),
        }
    }

    fn grid_paths<F>(get_point: F, u_steps: usize, v_steps: usize) -> Paths
    where
        F: Fn(usize, usize) -> Vector,
    {
        let mut paths = Paths::new();

        for v in 0..=v_steps {
            paths
                .new_path()
                .extend((0..=u_steps).map(|u| get_point(u, v)));
        }

        for u in 0..=u_steps {
            paths
                .new_path()
                .extend((0..=v_steps).map(|v| get_point(u, v)));
        }

        paths
    }

    pub fn mesh_from_grid(
        grid: Vec<Vector>,
        u_steps: usize,
        v_steps: usize,
        indexer: impl Fn(usize, usize) -> usize,
    ) -> Mesh {
        let (u_mapper, v_mapper) = {
            let upoints =
                |u| -> Vec<Vector> { (0..v_steps).map(|v| grid[indexer(u, v)]).collect() };
            let vpoints =
                |v| -> Vec<Vector> { (0..u_steps).map(|u| grid[indexer(u, v)]).collect() };
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

        let flipped = {
            let mut flipped = HashSet::new();
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
            flipped
        };

        Mesh::builder(grid, triangles)
            .flipped_triangles(flipped)
            .build()
    }
}

impl Shape for ParametricSurface {
    fn bounding_box(&self) -> BBox {
        self.mesh.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.mesh.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.mesh.intersect(r)
    }

    fn paths(&self, _args: &RenderArgs) -> Paths {
        self.paths.clone()
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

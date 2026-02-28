use crate::bounding_box::BBox;
use crate::hit::Hit;
use crate::mesh::Mesh;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::vector::Vector;
use std::collections::HashSet;

pub struct ParametricSurface {
    mesh: Mesh,
    paths: Paths<Vector>,
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

    fn grid_paths<F>(get_point: F, u_steps: usize, v_steps: usize) -> Paths<Vector>
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
                |u| -> Vec<Vector> { (0..=v_steps).map(|v| grid[indexer(u, v)]).collect() };
            let vpoints =
                |v| -> Vec<Vector> { (0..=u_steps).map(|u| grid[indexer(u, v)]).collect() };
            let [u0, ue] = [0, u_steps].map(upoints);
            let [v0, ve] = [0, v_steps].map(vpoints);
            let u_mapper = CyclicMapping::new_vector(&u0, &ue);
            let v_mapper = CyclicMapping::new_vector(&v0, &ve);
            (u_mapper, v_mapper)
        };

        let mut triangles = Vec::with_capacity(u_steps * v_steps * 6);
        let mut build_triangles = |u: usize, v: usize| -> (Option<usize>, Option<usize>) {
            let u_mapper = u_mapper.filter(|_| u == u_steps - 1);
            let v_mapper = v_mapper.filter(|_| v == v_steps - 1);

            let get_idx = |du: bool, dv: bool| {
                let mut curr_u = if du { u + 1 } else { u };
                let mut curr_v = if dv { v + 1 } else { v };

                if curr_u == u_steps {
                    if let Some(umap) = u_mapper {
                        curr_v = umap.map_index_inv(curr_v);
                        curr_u = 0;
                    }
                }
                // check the new `curr_v`
                if curr_v == v_steps {
                    if let Some(vmap) = v_mapper {
                        curr_u = vmap.map_index_inv(curr_u);
                        curr_v = 0;
                    }
                }

                indexer(curr_u, curr_v)
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
                |x0: &[Option<usize>], xe: &[Option<usize>], xmap: CyclicMapping| {
                    let steps = x0.len();
                    for (ib, b) in xe
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(i, ob)| ob.map(|b| (i, b)))
                    {
                        let mut p0 = xmap.map_index_inv(ib);
                        let mut p1 = xmap.map_index_inv(ib + 1);

                        if p0 == steps - 1 && p1 == 0 {
                            p1 = steps;
                        } else if p1 == steps - 1 && p0 == 0 {
                            p0 = steps;
                        }

                        if p0.abs_diff(p1) == 1 {
                            let ia = p0.min(p1);
                            if let Some(&Some(a)) = x0.get(ia) {
                                flipped.insert(if a < b { (a, b) } else { (b, a) });
                            }
                        }
                    }
                };

            if let (Some(uarr), Some(umap)) = (u_rev, u_mapper) {
                process_flipped(&uarr[..v_steps], &uarr[v_steps..], umap);
            }
            if let (Some(varr), Some(vmap)) = (v_rev, v_mapper) {
                process_flipped(&varr[..u_steps], &varr[u_steps..], vmap);
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

    fn paths(&self, _args: &RenderArgs) -> Paths<Vector> {
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
            CyclicMapping::Forward(0, _) => a_idx,
            CyclicMapping::Forward(offset, steps) => (offset + a_idx) % steps,
            CyclicMapping::Reverse(0, steps) => steps - a_idx,
            CyclicMapping::Reverse(offset, steps) => (offset + steps - (a_idx % steps)) % steps,
        }
    }

    #[inline]
    pub fn map_index_inv(&self, b_idx: usize) -> usize {
        match *self {
            CyclicMapping::Forward(0, _) => b_idx,
            CyclicMapping::Forward(offset, steps) => (b_idx + steps - (offset % steps)) % steps,
            CyclicMapping::Reverse(0, steps) => steps - b_idx,
            CyclicMapping::Reverse(offset, steps) => (offset + steps - (b_idx % steps)) % steps,
        }
    }

    pub fn new_eq<T>(a: &[T], b: &[T], eq: impl Fn(&T, &T) -> bool) -> Option<CyclicMapping> {
        let n = a.len();
        if n < 2 || a.len() != b.len() {
            return None;
        }
        let steps = n - 1;

        let is_closed = eq(&a[0], &a[steps]) && eq(&b[0], &b[steps]);
        let end = if is_closed { steps } else { 1 };

        for offset in 0..end {
            let forward_match = (0..n).all(|j| {
                let b_idx = if offset == 0 { j } else { (offset + j) % steps };
                eq(&a[j], &b[b_idx])
            });
            if forward_match {
                return Some(CyclicMapping::Forward(offset, steps));
            }

            let reverse_match = (0..n).all(|j| {
                let b_idx = if offset == 0 {
                    steps - j
                } else {
                    (offset + steps - (j % steps)) % steps
                };
                eq(&a[j], &b[b_idx])
            });
            if reverse_match {
                return Some(CyclicMapping::Reverse(offset, steps));
            }
        }
        None
    }

    pub fn new<T: Eq>(a: &[T], b: &[T]) -> Option<CyclicMapping> {
        Self::new_eq(a, b, |a, b| a == b)
    }

    pub fn new_vector(a: &[Vector], b: &[Vector]) -> Option<Self> {
        Self::new_eq(a, b, |a, &b| a.all_close(b))
    }
}

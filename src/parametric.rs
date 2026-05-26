use crate::bounding_box::BBox;
use crate::common::EPS;
use crate::hit::Hit;
use crate::mesh::Mesh;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::util::{ParamEdge, ParamPoint, trace_parametric_edge_curve};
use crate::vector::Vector;
use std::collections::HashSet;
use std::f64::consts::{SQRT_2, TAU};
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct SampledSwirlOffset {
    u_range: (f64, f64),
    v_range: (f64, f64),
    u_steps: usize,
    v_steps: usize,
    values: Vec<f64>,
}

impl fmt::Debug for SampledSwirlOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SampledSwirlOffset")
            .field("u_range", &self.u_range)
            .field("v_range", &self.v_range)
            .field("u_steps", &self.u_steps)
            .field("v_steps", &self.v_steps)
            .field("values", &format_args!("{} samples", self.values.len()))
            .finish()
    }
}

impl SampledSwirlOffset {
    pub fn new(
        values: Vec<f64>,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_steps: usize,
        v_steps: usize,
    ) -> Self {
        Self {
            u_range,
            v_range,
            u_steps,
            v_steps,
            values,
        }
    }

    fn grid_index(u: usize, v: usize, v_steps: usize) -> usize {
        u * (v_steps + 1) + v
    }

    fn normalized_step(value: f64, range: (f64, f64), steps: usize) -> f64 {
        if steps == 0 || (range.1 - range.0).abs() < EPS {
            return 0.0;
        }
        ((value - range.0) / (range.1 - range.0) * steps as f64).clamp(0.0, steps as f64)
    }

    fn sample(&self, u: usize, v: usize) -> f64 {
        let Some(expected_len) = (self.u_steps + 1).checked_mul(self.v_steps + 1) else {
            return 0.0;
        };
        if self.values.len() != expected_len {
            return 0.0;
        }
        self.values[Self::grid_index(u.min(self.u_steps), v.min(self.v_steps), self.v_steps)]
    }

    pub fn eval(&self, u: f64, v: f64) -> f64 {
        let u = Self::normalized_step(u, self.u_range, self.u_steps);
        let v = Self::normalized_step(v, self.v_range, self.v_steps);
        let u0 = if (u - self.u_steps as f64).abs() < EPS {
            self.u_steps.saturating_sub(1)
        } else {
            u.floor() as usize
        };
        let v0 = if (v - self.v_steps as f64).abs() < EPS {
            self.v_steps.saturating_sub(1)
        } else {
            v.floor() as usize
        };
        let u1 = (u0 + 1).min(self.u_steps);
        let v1 = (v0 + 1).min(self.v_steps);
        let fu = (u - u0 as f64).clamp(0.0, 1.0);
        let fv = (v - v0 as f64).clamp(0.0, 1.0);

        let a = self.sample(u0, v0) * (1.0 - fu) + self.sample(u1, v0) * fu;
        let b = self.sample(u0, v1) * (1.0 - fu) + self.sample(u1, v1) * fu;
        a * (1.0 - fv) + b * fv
    }
}

#[derive(Clone)]
pub enum SwirlOffset {
    Linear { twist: f64 },
    Function(Arc<dyn Fn(f64, f64) -> f64 + Send + Sync>),
    SampledGrid(SampledSwirlOffset),
}

impl fmt::Debug for SwirlOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linear { twist } => f.debug_struct("Linear").field("twist", twist).finish(),
            Self::Function(_) => f.write_str("Function(<callback>)"),
            Self::SampledGrid(grid) => f.debug_tuple("SampledGrid").field(grid).finish(),
        }
    }
}

impl Default for SwirlOffset {
    fn default() -> Self {
        Self::Linear {
            twist: ParametricSurfaceTexture::DEFAULT_SWIRL_TWIST,
        }
    }
}

impl SwirlOffset {
    pub fn linear(twist: f64) -> Self {
        Self::Linear { twist }
    }

    pub fn function<F>(offset: F) -> Self
    where
        F: Fn(f64, f64) -> f64 + Send + Sync + 'static,
    {
        Self::Function(Arc::new(offset))
    }

    pub fn sampled_grid(
        values: Vec<f64>,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_steps: usize,
        v_steps: usize,
    ) -> Self {
        Self::SampledGrid(SampledSwirlOffset::new(
            values, u_range, v_range, u_steps, v_steps,
        ))
    }

    fn eval(&self, u: f64, v: f64, radial_t: f64) -> f64 {
        match self {
            Self::Linear { twist } => TAU * twist * radial_t,
            Self::Function(offset) => offset(u, v),
            Self::SampledGrid(grid) => grid.eval(u, v),
        }
    }

    fn sample_turns(&self) -> f64 {
        match self {
            Self::Linear { twist } => twist.abs(),
            Self::Function(_) | Self::SampledGrid(_) => 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ParametricSurfaceTexture {
    #[default]
    Grid,
    Swirl {
        spacing: f64,
        offset: SwirlOffset,
    },
    Spiral {
        spacing: f64,
        arms: usize,
    },
}

impl ParametricSurfaceTexture {
    pub const DEFAULT_SPACING: f64 = 1.0;
    pub const DEFAULT_SWIRL_TWIST: f64 = 1.0;
    pub const DEFAULT_SPIRAL_ARMS: usize = 4;

    pub fn swirl() -> Self {
        Self::Swirl {
            spacing: Self::DEFAULT_SPACING,
            offset: SwirlOffset::default(),
        }
    }

    pub fn swirl_with_offset(spacing: f64, offset: SwirlOffset) -> Self {
        Self::Swirl { spacing, offset }
    }

    pub fn spiral() -> Self {
        Self::Spiral {
            spacing: Self::DEFAULT_SPACING,
            arms: Self::DEFAULT_SPIRAL_ARMS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParametricSurface {
    mesh: Mesh,
    grid: Vec<Vector>,
    u_steps: usize,
    v_steps: usize,
    u_range: (f64, f64),
    v_range: (f64, f64),
    pub texture: ParametricSurfaceTexture,
}

impl ParametricSurface {
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
        Self::from_grid_with_ranges(grid, u_steps, v_steps, u_range, v_range, indexer)
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
        Self::from_grid_with_ranges(
            grid,
            u_steps,
            v_steps,
            (0.0, u_steps as f64),
            (0.0, v_steps as f64),
            indexer,
        )
    }

    pub fn from_grid_with_ranges(
        grid: Vec<Vector>,
        u_steps: usize,
        v_steps: usize,
        u_range: (f64, f64),
        v_range: (f64, f64),
        indexer: impl Fn(usize, usize) -> usize,
    ) -> Self {
        let ordered_grid = Self::ordered_grid(&grid, u_steps, v_steps, &indexer);
        Self {
            mesh: Self::triangulate_grid(grid, u_steps, v_steps, indexer),
            grid: ordered_grid,
            u_steps,
            v_steps,
            u_range,
            v_range,
            texture: ParametricSurfaceTexture::default(),
        }
    }

    pub fn with_texture(mut self, texture: ParametricSurfaceTexture) -> Self {
        self.texture = texture;
        self
    }

    pub fn with_parameter_ranges(mut self, u_range: (f64, f64), v_range: (f64, f64)) -> Self {
        self.u_range = u_range;
        self.v_range = v_range;
        self
    }

    pub fn into_mesh(self) -> Mesh {
        self.mesh
    }

    fn ordered_grid(
        grid: &[Vector],
        u_steps: usize,
        v_steps: usize,
        indexer: &impl Fn(usize, usize) -> usize,
    ) -> Vec<Vector> {
        let mut ordered_grid = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
        for u in 0..=u_steps {
            for v in 0..=v_steps {
                ordered_grid.push(grid[indexer(u, v)]);
            }
        }
        ordered_grid
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

    #[inline]
    fn grid_index(u: usize, v: usize, v_steps: usize) -> usize {
        u * (v_steps + 1) + v
    }

    fn step_index(value: f64, steps: usize) -> Option<(usize, f64)> {
        if steps == 0 || value < -EPS || value > steps as f64 + EPS {
            return None;
        }

        let value = value.clamp(0.0, steps as f64);
        let index = if (value - steps as f64).abs() < EPS {
            steps - 1
        } else {
            value.floor() as usize
        };
        Some((index, value - index as f64))
    }

    fn edge_point(&self, edge: ParamEdge, point: ParamPoint) -> Vector {
        match edge {
            ParamEdge::U(u) => {
                let (v, t) = Self::step_index(point.v, self.v_steps).unwrap();
                let p0 = self.grid[Self::grid_index(u, v, self.v_steps)];
                let p1 = self.grid[Self::grid_index(u, v + 1, self.v_steps)];
                p0.mul_scalar(1.0 - t).add(p1.mul_scalar(t))
            }
            ParamEdge::V(v) => {
                let (u, t) = Self::step_index(point.u, self.u_steps).unwrap();
                let p0 = self.grid[Self::grid_index(u, v, self.v_steps)];
                let p1 = self.grid[Self::grid_index(u + 1, v, self.v_steps)];
                p0.mul_scalar(1.0 - t).add(p1.mul_scalar(t))
            }
            ParamEdge::Diagonal { u, v } => {
                let t = (point.v - v as f64).clamp(0.0, 1.0);
                let p10 = self.grid[Self::grid_index(u + 1, v, self.v_steps)];
                let p01 = self.grid[Self::grid_index(u, v + 1, self.v_steps)];
                p10.mul_scalar(1.0 - t).add(p01.mul_scalar(t))
            }
        }
    }

    fn trace_edge_curve(
        &self,
        samples: impl IntoIterator<Item = Option<ParamPoint>>,
    ) -> Paths<Vector> {
        trace_parametric_edge_curve(
            self.u_steps,
            self.v_steps,
            samples,
            |edge, point| self.edge_point(edge, point),
            |a, b| (*a).all_close(*b),
        )
    }

    fn texture_center_radius(&self) -> ((f64, f64), f64) {
        let u_max = self.u_steps as f64;
        let v_max = self.v_steps as f64;
        ((u_max * 0.5, v_max * 0.5), u_max.max(v_max) * 0.5 * SQRT_2)
    }

    fn texture_spacing(spacing: f64) -> f64 {
        if spacing.is_finite() && spacing > EPS {
            spacing
        } else {
            ParametricSurfaceTexture::DEFAULT_SPACING
        }
    }

    fn radial_sample_count(radius: f64, spacing: f64, angular_turns: f64) -> usize {
        let max_step = spacing.clamp(0.05, 0.5);
        let length = radius * (1.0 + (TAU * angular_turns).abs());
        (length / max_step).ceil().max(2.0) as usize
    }

    fn spiral_sample_count(radius: f64, spacing: f64, turns: f64) -> usize {
        let max_step = spacing.clamp(0.05, 0.5);
        let length = radius + radius * TAU * turns * 0.5;
        (length / max_step).ceil().max(2.0) as usize
    }

    fn parameter_at_step(value: f64, steps: usize, range: (f64, f64)) -> f64 {
        if steps == 0 {
            range.0
        } else {
            range.0 + value / steps as f64 * (range.1 - range.0)
        }
    }

    fn parameter_at_grid_point(&self, u: f64, v: f64) -> (f64, f64) {
        (
            Self::parameter_at_step(u, self.u_steps, self.u_range),
            Self::parameter_at_step(v, self.v_steps, self.v_range),
        )
    }

    fn paths_swirl(&self, spacing: f64, offset: &SwirlOffset) -> Paths<Vector> {
        let spacing = Self::texture_spacing(spacing);

        let ((center_u, center_v), radius) = self.texture_center_radius();
        let sample_count = Self::radial_sample_count(radius, spacing, offset.sample_turns());
        let rays = (TAU * radius / spacing).ceil().max(1.0) as usize;
        let mut paths = Paths::new();

        for ray in 0..rays {
            let base_angle = ray as f64 / rays as f64 * TAU;
            let samples = (0..=sample_count).map(|i| {
                let radial_t = i as f64 / sample_count as f64;
                let r = radius * radial_t;
                let u0 = center_u + base_angle.cos() * r;
                let v0 = center_v + base_angle.sin() * r;
                let (param_u, param_v) = self.parameter_at_grid_point(u0, v0);
                let angle = base_angle + offset.eval(param_u, param_v, radial_t);
                let u = center_u + angle.cos() * r;
                let v = center_v + angle.sin() * r;

                Some(ParamPoint::new(u, v))
            });

            paths.extend(self.trace_edge_curve(samples));
        }

        paths
    }

    fn paths_spiral(&self, spacing: f64, arms: usize) -> Paths<Vector> {
        let spacing = Self::texture_spacing(spacing);
        let ((center_u, center_v), radius) = self.texture_center_radius();
        let arms = arms.max(1);
        let turns = (radius / spacing).ceil().max(1.0);
        let sample_count = Self::spiral_sample_count(radius, spacing, turns);
        let mut paths = Paths::new();

        for arm in 0..arms {
            let phase = arm as f64 / arms as f64 * TAU;
            let samples = (0..=sample_count).map(|i| {
                let t = i as f64 / sample_count as f64;
                let r = radius * (1.0 - t);
                let angle = phase + t * TAU * turns;
                let u = center_u + angle.cos() * r;
                let v = center_v + angle.sin() * r;

                Some(ParamPoint::new(u, v))
            });

            paths.extend(self.trace_edge_curve(samples));
        }

        paths
    }

    fn triangulate_grid(
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

                if curr_u == u_steps
                    && let Some(umap) = u_mapper
                {
                    curr_v = umap.map_index_inv(curr_v);
                    curr_u = 0;
                }
                // check the new `curr_v`
                if curr_v == v_steps
                    && let Some(vmap) = v_mapper
                {
                    curr_u = vmap.map_index_inv(curr_u);
                    curr_v = 0;
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
        match &self.texture {
            ParametricSurfaceTexture::Grid => Self::grid_paths(
                |u, v| self.grid[Self::grid_index(u, v, self.v_steps)],
                self.u_steps,
                self.v_steps,
            ),
            ParametricSurfaceTexture::Swirl { spacing, offset } => {
                self.paths_swirl(*spacing, offset)
            }
            ParametricSurfaceTexture::Spiral { spacing, arms } => {
                self.paths_spiral(*spacing, *arms)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn flat_surface(u_steps: usize, v_steps: usize) -> ParametricSurface {
        let mut grid = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
        for u in 0..=u_steps {
            for v in 0..=v_steps {
                grid.push(Vector::new(u as f64, v as f64, 0.0));
            }
        }
        ParametricSurface::from_grid(grid, u_steps, v_steps, move |u, v| u * (v_steps + 1) + v)
    }

    fn covered_triangles(
        paths: &Paths<Vector>,
        u_steps: usize,
        v_steps: usize,
    ) -> HashSet<(usize, usize, bool)> {
        let mut covered = HashSet::new();
        for path in paths.iter_paths() {
            for segment in path.windows(2) {
                let mid = segment[0].add(segment[1]).mul_scalar(0.5);
                if mid.x <= 0.0
                    || mid.y <= 0.0
                    || mid.x >= u_steps as f64
                    || mid.y >= v_steps as f64
                {
                    continue;
                }

                let u = mid.x.floor() as usize;
                let v = mid.y.floor() as usize;
                let s = mid.x - u as f64;
                let t = mid.y - v as f64;
                covered.insert((u, v, s + t > 1.0));
            }
        }
        covered
    }

    #[test]
    fn dense_swirl_covers_a_small_grid() {
        let surface = flat_surface(4, 4);
        let paths = surface.paths_swirl(0.25, &SwirlOffset::linear(1.0));
        let covered = covered_triangles(&paths, 4, 4);

        assert_eq!(covered.len(), 4 * 4 * 2);
    }

    #[test]
    fn swirl_function_offset_uses_original_parameter_ranges() {
        let surface = ParametricSurface::new(
            |u, v| Vector::new(u, v, 0.0),
            (-3.0, 3.0),
            (-2.0, 4.0),
            4,
            6,
        );

        assert_eq!(surface.parameter_at_grid_point(0.0, 0.0), (-3.0, -2.0));
        assert_eq!(surface.parameter_at_grid_point(2.0, 3.0), (0.0, 1.0));
        assert_eq!(surface.parameter_at_grid_point(4.0, 6.0), (3.0, 4.0));
    }

    #[test]
    fn sampled_swirl_offset_interpolates_values() {
        let offset =
            SampledSwirlOffset::new(vec![0.0, 2.0, 4.0, 6.0], (0.0, 1.0), (0.0, 1.0), 1, 1);

        assert!((offset.eval(0.5, 0.5) - 3.0).abs() < EPS);
    }

    #[test]
    fn dense_spiral_covers_a_small_grid() {
        let surface = flat_surface(4, 4);
        let paths = surface.paths_spiral(0.25, 4);
        let covered = covered_triangles(&paths, 4, 4);

        assert_eq!(covered.len(), 4 * 4 * 2);
    }
}

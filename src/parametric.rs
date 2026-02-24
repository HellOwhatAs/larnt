use crate::bounding_box::Box;
use crate::hit::Hit;
use crate::mesh::Mesh;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::vector::Vector;

pub struct ParametricSurface {
    mesh: Mesh,
    paths: Paths,
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
        let get_point = |i: usize, j: usize| grid[i * (v_steps + 1) + j];

        Self::from_grid(get_point, u_steps, v_steps)
    }

    pub fn from_grid(
        get_point: impl Fn(usize, usize) -> Vector,
        u_steps: usize,
        v_steps: usize,
    ) -> Self {
        Self {
            mesh: Mesh::parametric_surface(&get_point, 0..u_steps, 0..v_steps),
            paths: Self::grid_paths(get_point, u_steps, v_steps),
        }
    }

    fn grid_paths<F>(get_point: F, u_steps: usize, v_steps: usize) -> Paths
    where
        F: Fn(usize, usize) -> Vector,
    {
        let mut raw_paths = Vec::with_capacity(u_steps + 1 + v_steps + 1);

        for v in 0..=v_steps {
            let mut path = Vec::with_capacity(u_steps + 1);
            for u in 0..=u_steps {
                path.push(get_point(u, v));
            }
            raw_paths.push(path);
        }

        for u in 0..=u_steps {
            let mut path = Vec::with_capacity(v_steps + 1);
            for v in 0..=v_steps {
                path.push(get_point(u, v));
            }
            raw_paths.push(path);
        }

        Paths::from_vec(raw_paths)
    }
}

impl Shape for ParametricSurface {
    fn compile(&mut self) {
        self.mesh.compile();
    }

    fn bounding_box(&self) -> Box {
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

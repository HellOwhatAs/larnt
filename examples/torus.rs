use larnt::{ParametricSurface, Vector, render};
use std::f64::consts::PI;

fn main() {
    let torus_func = |u: f64, v: f64| -> Vector {
        let x = (1.5 + 0.5 * v.cos()) * u.cos();
        let y = (1.5 + 0.5 * v.cos()) * u.sin();
        let z = 0.5 * v.sin();
        Vector::new(x, y, z)
    };
    let torus_mesh = ParametricSurface::new(torus_func, (0.0, 2.0 * PI), (0.0, 2.0 * PI), 64, 32);
    render(vec![torus_mesh])
        .eye(Vector::new(3., 3., 3.))
        .call()
        .write_to_png("out.png", 1024.0, 1024.0);
}

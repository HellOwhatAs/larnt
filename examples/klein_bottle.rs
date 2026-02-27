use larnt::{ParametricSurface, Vector, render};
use std::f64::consts::PI;

fn main() {
    let radius = 2.0;
    let klein_func = |u: f64, v: f64| -> Vector {
        let sin_u_half = (u / 2.0).sin();
        let cos_u_half = (u / 2.0).cos();
        let sin_v = v.sin();
        let sin_2v = (2.0 * v).sin();

        let x = (radius + cos_u_half * sin_v - sin_u_half * sin_2v) * u.cos();
        let y = (radius + cos_u_half * sin_v - sin_u_half * sin_2v) * u.sin();
        let z = sin_u_half * sin_v + cos_u_half * sin_2v;
        Vector::new(x, y, z)
    };
    let torus_mesh = ParametricSurface::new(klein_func, (0.0, 2.0 * PI), (0.0, 2.0 * PI), 128, 64);

    render(vec![torus_mesh])
        .eye(Vector::new(5., 0.5, 5.))
        .call()
        .write_to_png("out.png", 1024.0, 1024.0);
}

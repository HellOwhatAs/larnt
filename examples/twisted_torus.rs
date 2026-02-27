use larnt::{ParametricSurface, Vector, render};
use std::f64::consts::PI;

fn main() {
    let radius = 2.0;
    let tube_radius = 0.6;

    let twisted_torus_func = move |u: f64, v: f64| -> Vector {
        let k = 1.0;
        let v_shifted = v + k * u;

        let x = (radius + tube_radius * v_shifted.cos()) * u.cos();
        let y = (radius + tube_radius * v_shifted.cos()) * u.sin();
        let z = tube_radius * v_shifted.sin();
        Vector::new(x, y, z)
    };

    let twisted_torus =
        ParametricSurface::new_mesh(twisted_torus_func, (0.0, 2.0 * PI), (0.0, 2.0 * PI), 20, 10);

    render(vec![twisted_torus])
        .eye(Vector::new(3.5, 3.5, 3.5))
        .call()
        .write_to_png("out.png", 1024.0, 1024.0);
}

use std::f64::consts::PI;

use larnt::{ParametricSurface, Scene, Vector};

fn main() {
    let r = 2.0;
    let width = 1.0;
    let thickness = 0.1;

    let solid_mobius_func = |u: f64, v: f64| -> Vector {
        let cx = width * v.cos();
        let cz = thickness * v.sin();

        let twist = u / 2.0;
        let rx = cx * twist.cos() - cz * twist.sin();
        let rz = cx * twist.sin() + cz * twist.cos();

        let x = (r + rx) * u.cos();
        let y = (r + rx) * u.sin();
        let z = rz;
        Vector::new(x, y, z)
    };
    let torus_mesh = ParametricSurface::new(solid_mobius_func, (0.0, 4.0 * PI), (0.0, PI), 160, 20);
    let mut scene = Scene::new();
    scene.add(torus_mesh);
    scene
        .render(Vector::new(4., 4., 4.))
        .call()
        .write_to_png("out.png", 1024.0, 1024.0);
}

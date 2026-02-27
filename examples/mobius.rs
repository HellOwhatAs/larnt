use larnt::{ParametricSurface, Primitive, Vector, mesh::MeshTexture, render};
use std::f64::consts::PI;

fn main() {
    let radius = 2.0;
    let width = 0.5;

    let mobius_func = |u: f64, v: f64| -> Vector {
        let x = (radius + (v / 2.0) * (u / 2.0).cos()) * u.cos();
        let y = (radius + (v / 2.0) * (u / 2.0).cos()) * u.sin();
        let z = (v / 2.0) * (u / 2.0).sin();
        Vector::new(x, y, z)
    };
    let mut mobius =
        ParametricSurface::new_mesh(mobius_func, (0.0, 2.0 * PI), (-width, width), 80, 20);
    mobius.texture = MeshTexture::Silhouette;

    let mobius_func2 = |u: f64, v: f64| -> Vector {
        let x = (radius + (v / 2.0) * (u / 2.0).cos()) * u.cos() - 2.0;
        let z = (radius + (v / 2.0) * (u / 2.0).cos()) * u.sin();
        let y = (v / 2.0) * (u / 2.0).sin();
        Vector::new(x, y, z)
    };
    let mut mobius2 =
        ParametricSurface::new_mesh(mobius_func2, (0.0, 2.0 * PI), (-width, width), 80, 20);
    mobius2.texture = MeshTexture::Silhouette;

    render::<Primitive>(vec![mobius.into(), mobius2.into()])
        .eye(Vector::new(3., -5., 1.))
        .center(Vector::new(-0.2, 0., 0.))
        .call()
        .write_to_png("out.png", 1024.0, 1024.0);
}

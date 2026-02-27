use larnt::{
    Matrix, ParametricSurface, Primitive, TransformedShape, Vector, mesh::MeshTexture, render,
};
use std::f64::consts::PI;

fn main() {
    let radius = 1.5;
    let tube_radius = 0.5;
    let torus_func = |u: f64, v: f64| -> Vector {
        let x = (radius + tube_radius * v.cos()) * u.cos();
        let y = (radius + tube_radius * v.cos()) * u.sin();
        let z = tube_radius * v.sin();
        Vector::new(x, y, z)
    };
    let twisted_func = move |u: f64, v: f64| -> Vector {
        let k = 1.0;
        let v_shifted = v + k * u;

        let x = (radius + tube_radius * v_shifted.cos()) * u.cos();
        let y = (radius + tube_radius * v_shifted.cos()) * u.sin();
        let z = tube_radius * v_shifted.sin();
        Vector::new(x, y, z)
    };

    let range = (0.0, 2. * PI);
    let torus = ParametricSurface::new(torus_func, range, range, 64, 32);
    let twisted = ParametricSurface::new_mesh(twisted_func, range, range, 20, 10);
    let mut silhouette = ParametricSurface::new_mesh(torus_func, range, range, 64, 32);
    silhouette.texture = MeshTexture::Silhouette;

    let offset = radius + tube_radius;
    render::<Primitive>(vec![
        TransformedShape::new(
            torus.into(),
            Matrix::translate(Vector::new(offset, 0., 0.))
                .rotated(Vector::new(1., 0., 0.), PI / 2.0),
        )
        .into(),
        twisted.into(),
        TransformedShape::new(
            silhouette.into(),
            Matrix::translate(Vector::new(-offset, 0., 0.))
                .rotated(Vector::new(1., 0., 0.), PI / 2.0),
        )
        .into(),
    ])
    .eye(Vector::new(2., 7., 5.))
    .center(Vector::new(0.3, 0., 0.))
    .call()
    .write_to_png("out.png", 1024.0, 1024.0);
}

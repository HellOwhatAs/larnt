use larnt::{ParametricSurface, Primitive, Vector, mesh::MeshTexture, render};

fn main() {
    let func = |u: f64, v: f64| -> Vector {
        let z = 2.0 * (u.powi(2) + v.powi(2)).sqrt().sin();
        Vector::new(u, v, z)
    };
    let range = (-20.0, 20.0);
    let surface = ParametricSurface::new(func, range, range, 100, 100);
    let mut mesh = ParametricSurface::new_mesh(
        |u, v| func(u, v).add(Vector::new(-6., -2., 20.)),
        range,
        range,
        200,
        200,
    );
    mesh.texture = MeshTexture::silhouette().call();

    render::<Primitive>(vec![surface.into(), mesh.into()])
        .eye(Vector::new(75., 35., 50.))
        .center(Vector::new(2., 0., 10.))
        .fovy(32.)
        .call()
        .to_image(1024.0, 1024.0)
        .linewidth(1.5)
        .call()
        .save("out.png")
        .expect("Failed to save image");
}

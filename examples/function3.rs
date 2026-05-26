use larnt::{
    ParametricSurface, ParametricSurfaceTexture, Primitive, Vector, mesh::MeshTexture, render,
};

fn main() {
    let func = |x: f64, y: f64| -> Vector {
        let z = (-1. / (x * x + y * y)).max(-10.0);
        Vector::new(x, y, z)
    };
    let range = (-3.0, 3.0);
    let surface = ParametricSurface::new(func, range, range, 100, 100).with_texture(
        ParametricSurfaceTexture::Swirl {
            spacing: 10.,
            twist: -1.0,
        },
    );

    render(vec![surface])
        .eye(Vector::new(3., 0., 3.))
        .center(Vector::new(1.1, 0., 0.))
        .call()
        .to_image(1024.0, 1024.0)
        .linewidth(1.5)
        .call()
        .save("out.png")
        .expect("Failed to save image");
}

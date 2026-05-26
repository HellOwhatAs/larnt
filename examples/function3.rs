use larnt::{ParametricSurface, ParametricSurfaceTexture, SwirlOffset, Vector, render};

fn main() {
    let func = |x: f64, y: f64| -> Vector {
        let z = (-1. / (x * x + y * y)).max(-1e5);
        Vector::new(x, y, z)
    };
    let range = (-3.0, 3.0);
    let surface = ParametricSurface::new(func, range, range, 1024, 1024).with_texture(
        ParametricSurfaceTexture::Swirl {
            spacing: 80.0,
            offset: SwirlOffset::function(|x, y| {
                let z = (-1. / (x * x + y * y)).max(-1e5);
                (-z).max(0.0).powf(1.4)
            }),
        },
    );

    render(vec![surface])
        .eye(Vector::new(3., 0., 3.))
        .center(Vector::new(1.1, 0., 0.))
        .width(1024.0 * 3.0)
        .height(1024.0 * 3.0)
        .call()
        .to_image(1024.0 * 3.0, 1024.0 * 3.0)
        .linewidth(4.5)
        .call()
        .save("out.png")
        .expect("Failed to save image");
}

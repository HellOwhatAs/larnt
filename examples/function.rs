use larnt::{ParametricSurface, Primitive, Sphere, SphereTexture, Vector, render};

fn main() {
    let mut shapes: Vec<Primitive> = Vec::new();

    shapes.push(Primitive::Dynamic(Box::new(
        ParametricSurface::new(
            |x, y| Vector::new(x, y, x * y),
            (-1.0, 1.0),
            (-1.0, 1.0),
            100,
            100,
        )
        .with_texture(larnt::ParametricSurfaceTexture::Spiral {
            spacing: 3.0,
            arms: 2,
        }),
    )));
    shapes.push(Primitive::Dynamic(Box::new(ParametricSurface::new(
        |x, y| Vector::new(x, y, 0.0),
        (-1.0, 1.0),
        (-1.0, 1.0),
        20,
        20,
    ))));
    shapes.push(
        Sphere::builder(Vector::new(0.0, -0.6, 0.0), 0.25)
            .texture(SphereTexture::random_circles(42).call())
            .build()
            .into(),
    );

    let eye = Vector::new(3.0, 0.5, 3.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = render(shapes)
        .eye(eye)
        .width(width)
        .height(height)
        .fovy(40.0)
        .call();
    paths
        .to_image(width, height)
        .linewidth(1.5)
        .call()
        .save("out.png")
        .expect("Failed to save image");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

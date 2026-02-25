use larnt::{BBox, Function, FunctionTexture, Primitive, Sphere, SphereTexture, Vector, render};

fn main() {
    let mut shapes: Vec<Primitive> = Vec::new();
    let bbox = BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0));

    shapes.push(Primitive::Dynamic(Box::new(
        Function::builder(|x, y| x * y, bbox)
            .step(0.01)
            .texture(FunctionTexture::Spiral)
            .build(),
    )));
    shapes.push(Primitive::Dynamic(Box::new(
        Function::builder(|_, _| 0.0, bbox).step(0.01).build(),
    )));
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

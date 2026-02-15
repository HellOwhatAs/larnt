use larnt::{Box as BBox, Function, FunctionTexture, Scene, Sphere, SphereTexture, Vector};

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0));

    scene.add(
        Function::builder(|x, y| x * y, bbox)
            .step(0.01)
            .texture(FunctionTexture::Spiral)
            .build(),
    );
    scene.add(Function::builder(|_, _| 0.0, bbox).step(0.01).build());
    scene.add(
        Sphere::builder(Vector::new(0.0, -0.6, 0.0), 0.25)
            .texture(SphereTexture::random_circles(42).call())
            .build(),
    );

    let eye = Vector::new(3.0, 0.5, 3.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene
        .render(eye)
        .width(width)
        .height(height)
        .fovy(40.0)
        .call();
    paths
        .to_image(width, height, 1.5)
        .save("out.png")
        .expect("Failed to save image");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

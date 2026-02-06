use larnt::{Box as BBox, Direction, Function, FunctionTexture, Scene, Sphere, Vector};

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0));

    scene.add(
        Function::new(|x, y| x * y, bbox, Direction::Below, 0.1)
            .with_texture(FunctionTexture::Spiral),
    );
    scene.add(
        Function::new(|_, _| 0.0, bbox, Direction::Below, 0.1).with_texture(FunctionTexture::Grid),
    );
    scene.add(
        Sphere::new(Vector::new(0.0, -0.6, 0.0), 0.25)
            .with_texture(larnt::SphereTexture::RandomCircles(42)),
    );

    let eye = Vector::new(3.0, 0.5, 3.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene.render(eye, center, up, width, height, 40.0, 0.1, 100.0, 0.1);
    paths
        .to_image(width, height, 1.5)
        .save("out.png")
        .expect("Failed to save image");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

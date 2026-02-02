use ln::{Box as BBox, Direction, Function, FunctionTexture, Scene, Sphere, Vector};

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0));

    scene.add(
        Function::new(|x, y| x * y, bbox, Direction::Below).with_texture(FunctionTexture::Spiral),
    );

    scene
        .add(Function::new(|_, _| 0.0, bbox, Direction::Below).with_texture(FunctionTexture::Grid));

    scene.add(
        Sphere::new(Vector::new(2.0, 0.25, 0.5), 0.6)
            .with_texture(ln::SphereTexture::RandomCircles(42)),
    );

    let eye = Vector::new(3.75, 1.75, 5.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene.render(eye, center, up, width, height, 50.0, 0.1, 100.0, 0.1);
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

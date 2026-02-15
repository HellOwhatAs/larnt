use larnt::{Box as BBox, Function, Scene, Vector};

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(
        Vector::new(-25.0, -25.0, -20.0),
        Vector::new(25.0, 25.0, 10.0),
    );

    scene.add(
        Function::builder(
            |x, y| (x).sin() * (y).cos() - (x.powi(2) + y.powi(2)) * 0.01,
            bbox,
        )
        .build(),
    );

    let a = std::f64::consts::PI / 4.0;
    let eye = Vector::new(a.cos() * 28.0, a.sin() * 28.0, 10.0);
    let center = Vector::new(a.cos() * 9.0, a.sin() * 9.0, -4.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene
        .render(eye)
        .center(center)
        .width(width)
        .height(height)
        .fovy(70.0)
        .call();
    paths
        .to_image(width, height, 0.8)
        .save("out.png")
        .expect("Failed to save image");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

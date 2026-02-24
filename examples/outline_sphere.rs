use larnt::{Scene, Sphere, Vector};

fn main() {
    let eye = Vector::new(1.0, 1.0, 1.0);
    let mut scene = Scene::new();
    scene.add(Sphere::builder(eye.mul_scalar(-1.0), 1.0).build());
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene
        .render(eye)
        .width(width)
        .height(height)
        .step(1e-3)
        .call();
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

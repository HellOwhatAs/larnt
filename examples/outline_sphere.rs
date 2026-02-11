use larnt::{OutlineSphere, Scene, Vector};

fn main() {
    let eye = Vector::new(1.0, 1.0, 1.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);

    let mut scene = Scene::new();
    scene.add(OutlineSphere::new(eye, up, eye.mul_scalar(-10.0), 1.0));
    let width = 1024.0;
    let height = 1024.0;
    let fovy = 50.0;

    let paths = scene.render(eye, center, up, width, height, fovy, 0.1, 100.0, 150.0);
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

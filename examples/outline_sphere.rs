use larnt::{Sphere, Vector, render};

fn main() {
    let eye = Vector::new(1.0, 1.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = render(vec![Sphere::builder(eye.mul_scalar(-1.0), 1.0).build()])
        .eye(eye)
        .width(width)
        .height(height)
        .step(1e-3)
        .call();
    paths
        .write_to_png("out.png", width, height)
        .expect("Failed to write PNG");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

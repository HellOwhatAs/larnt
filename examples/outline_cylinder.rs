use larnt::{Cylinder, Vector, render};

fn main() {
    // define rendering parameters
    let width = 1024.0; // rendered width
    let height = 1024.0; // rendered height

    // compute 2D paths that depict the 3D scene
    let paths = render(vec![Cylinder::builder(1.0, 0.0, 1.0).build()])
        .eye(Vector::new(4.0, 3.0, 4.0))
        .width(width)
        .height(height)
        .step(1e-3)
        .call();

    // save the result as a png
    paths
        .write_to_png("out.png", width, height)
        .expect("Failed to write PNG");

    // save the result as an svg
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

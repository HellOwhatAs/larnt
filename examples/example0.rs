use larnt::{Cube, Vector, render};

fn main() {
    // create a scene and add a single cube
    let mut shapes = Vec::new();
    shapes.push(Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build());

    let (width, height) = (1024.0, 1024.0);

    // compute 2D paths that depict the 3D scene
    let paths = render(shapes)
        .eye(Vector::new(4.0, 3.0, 2.0))
        .width(width)
        .height(height)
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

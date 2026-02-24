use larnt::{Cone, Scene, Vector};

fn main() {
    // create a scene and add a single cube
    let mut scene = Scene::new();

    // define rendering parameters
    let width = 1024.0; // rendered width
    let height = 1024.0; // rendered height

    scene.add(Cone::builder(1.0, 1.0).build());

    // compute 2D paths that depict the 3D scene
    let eye = Vector::new(4.0, 3.0, 6.5);
    let paths = scene
        .render(eye)
        .width(width)
        .height(height)
        .step(1e-3)
        .call();

    // save the result as a png
    paths.write_to_png("out.png", width, height);

    // save the result as an svg
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

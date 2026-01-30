use ln::{Cube, Scene, Vector};

fn main() {
    // create a scene and add a single cube
    let mut scene = Scene::new();
    scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));

    // define camera parameters
    let eye = Vector::new(4.0, 3.0, 2.0);    // camera position
    let center = Vector::new(0.0, 0.0, 0.0); // camera looks at
    let up = Vector::new(0.0, 0.0, 1.0);     // up direction

    // define rendering parameters
    let width = 1024.0;  // rendered width
    let height = 1024.0; // rendered height
    let fovy = 50.0;     // vertical field of view, degrees
    let znear = 0.1;     // near z plane
    let zfar = 10.0;     // far z plane
    let step = 0.01;     // how finely to chop the paths for visibility testing

    // compute 2D paths that depict the 3D scene
    let paths = scene.render(eye, center, up, width, height, fovy, znear, zfar, step);

    // save the result as a png
    paths.write_to_png("out.png", width, height);

    // save the result as an svg
    paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
}

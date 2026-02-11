use larnt::{Matrix, Scene, TransformedShape, Vector, load_obj};
use std::sync::Arc;

fn main() {
    let mut scene = Scene::new();
    let mesh = load_obj("examples/suzanne.obj")
        .expect("Failed to load OBJ")
        .unit_cube();

    let transform = Matrix::rotate(Vector::new(0.0, 1.0, 0.0), 0.5);
    scene.add_arc(Arc::new(TransformedShape::new(Arc::new(mesh), transform)));

    let eye = Vector::new(-0.5, 0.5, 2.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene.render(eye, center, up, width, height, 35.0, 0.1, 100.0, 1.0);
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

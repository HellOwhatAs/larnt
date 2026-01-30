use ln::{load_obj, Matrix, Scene, TransformedShape, Vector};
use std::sync::Arc;

fn main() {
    let mut scene = Scene::new();
    let mut mesh = load_obj("examples/suzanne.obj").expect("Failed to load OBJ");
    mesh.unit_cube();
    
    let transform = Matrix::rotate(Vector::new(0.0, 1.0, 0.0), 0.5);
    scene.add_arc(Arc::new(TransformedShape::new(Arc::new(mesh), transform)));
    
    let eye = Vector::new(-0.5, 0.5, 2.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let width = 1024.0;
    let height = 1024.0;
    
    let paths = scene.render(eye, center, up, width, height, 35.0, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
}

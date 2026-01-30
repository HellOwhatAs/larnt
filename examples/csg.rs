use ln::{new_difference, new_intersection, radians, Cylinder, Matrix, Scene, Shape, Sphere, TransformedShape, Vector};
use std::sync::Arc;

fn main() {
    let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 1.0));
    let cube: Arc<dyn Shape + Send + Sync> = Arc::new(ln::Cube::new(
        Vector::new(-0.8, -0.8, -0.8),
        Vector::new(0.8, 0.8, 0.8),
    ));
    
    let cyl1: Arc<dyn Shape + Send + Sync> = Arc::new(Cylinder::new(0.4, -2.0, 2.0));
    let cyl2: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
        Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
        Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
    ));
    let cyl3: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
        Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
        Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
    ));
    
    let shape = new_difference(vec![
        new_intersection(vec![sphere, cube]),
        cyl1,
        cyl2,
        cyl3,
    ]);
    
    for i in (0..90).step_by(2) {
        println!("{}", i);
        let mut scene = Scene::new();
        let m = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(i as f64));
        scene.add_arc(Arc::new(TransformedShape::new(Arc::clone(&shape), m)));
        
        let eye = Vector::new(0.0, 6.0, 2.0);
        let center = Vector::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 0.0, 1.0);
        let width = 750.0;
        let height = 750.0;
        
        let paths = scene.render(eye, center, up, width, height, 20.0, 0.1, 100.0, 0.01);
        paths.write_to_png(&format!("out{:03}.png", i), width, height);
    }
}

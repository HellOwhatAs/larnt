use ln::{Sphere, Scene, Vector};

fn main() {
    let mut scene = Scene::new();
    let n = 8;
    
    for x in -n..=n {
        for y in -n..=n {
            scene.add(Sphere::new(Vector::new(x as f64, y as f64, 0.0), 0.45));
        }
    }
    
    let eye = Vector::new(8.0, 8.0, 1.0);
    let center = Vector::new(0.0, 0.0, -4.25);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;
    
    let paths = scene.render(eye, center, up, width, height, 50.0, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
}

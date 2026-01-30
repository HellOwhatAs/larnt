use ln::{OutlineSphere, Scene, Vector};
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let eye = Vector::new(8.0, 8.0, 8.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    
    let mut scene = Scene::new();
    let n = 10;
    
    for x in -n..=n {
        for y in -n..=n {
            let z = rng.gen::<f64>() * 3.0;
            let v = Vector::new(x as f64, y as f64, z);
            let sphere = OutlineSphere::new(eye, up, v, 0.45);
            scene.add(sphere);
        }
    }
    
    let width = 1920.0;
    let height = 1200.0;
    let fovy = 50.0;
    
    let paths = scene.render(eye, center, up, width, height, fovy, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
}

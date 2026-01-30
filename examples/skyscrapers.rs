use ln::{Cube, Scene, Vector};
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let mut scene = Scene::new();
    let n = 15;
    
    for x in -n..=n {
        for y in -n..=n {
            let p = rng.gen::<f64>() * 0.25 + 0.2;
            let fx = x as f64;
            let fy = y as f64;
            let fz = rng.gen::<f64>() * 3.0 + 1.0;
            
            // Skip one building to create a gap (matching original example)
            if x == 2 && y == 1 {
                continue;
            }
            
            let shape = Cube::new(
                Vector::new(fx - p, fy - p, 0.0),
                Vector::new(fx + p, fy + p, fz),
            );
            scene.add(shape);
        }
    }
    
    let eye = Vector::new(1.75, 1.25, 6.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;
    
    let paths = scene.render(eye, center, up, width, height, 100.0, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
}

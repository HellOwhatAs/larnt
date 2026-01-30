use ln::{Cube, Scene, Vector};
use rand::Rng;

fn make_cube(x: f64, y: f64, z: f64) -> Cube {
    let size = 0.5;
    let v = Vector::new(x, y, z);
    Cube::new(v.sub_scalar(size), v.add_scalar(size))
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut scene = Scene::new();
    
    for x in -2..=2 {
        for y in -2..=2 {
            let z = rng.gen::<f64>();
            scene.add(make_cube(x as f64, y as f64, z));
        }
    }
    
    let eye = Vector::new(6.0, 5.0, 3.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1920.0;
    let height = 1200.0;
    let fovy = 30.0;
    
    let paths = scene.render(eye, center, up, width, height, fovy, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
    paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
}

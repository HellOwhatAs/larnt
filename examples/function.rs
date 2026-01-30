use ln::{Direction, Function, Scene, Vector, Box as BBox};

fn my_function(x: f64, y: f64) -> f64 {
    -1.0 / (x * x + y * y)
}

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(Vector::new(-2.0, -2.0, -4.0), Vector::new(2.0, 2.0, 2.0));
    scene.add(Function::new(my_function, bbox, Direction::Below));
    
    let eye = Vector::new(3.0, 0.0, 3.0);
    let center = Vector::new(1.1, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;
    
    let paths = scene.render(eye, center, up, width, height, 50.0, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
    paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
}

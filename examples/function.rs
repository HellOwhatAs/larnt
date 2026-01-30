use ln::{Direction, Function, FunctionTexture, Scene, Vector, Box as BBox};

fn my_function(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

fn main() {
    let mut scene = Scene::new();
    let bbox = BBox::new(Vector::new(-3.0, -3.0, -1.0), Vector::new(3.0, 3.0, 1.0));
    
    // Use grid texture (default) - works with any function
    // Other options: FunctionTexture::Swirl (best for negative z), FunctionTexture::Spiral
    scene.add(Function::new(my_function, bbox, Direction::Below).with_texture(FunctionTexture::Grid));
    
    let eye = Vector::new(4.0, 3.0, 2.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;
    
    let paths = scene.render(eye, center, up, width, height, 50.0, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
    paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
}

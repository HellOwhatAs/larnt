use larnt::{Cube, Scene, Vector};
use rand::{Rng, SeedableRng, rngs::SmallRng};

fn make_cube(x: f64, y: f64, z: f64) -> Cube {
    let size = 0.5;
    let v = Vector::new(x, y, z);
    Cube::builder(v.sub_scalar(size), v.add_scalar(size)).build()
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(42);
    let mut scene = Scene::new();

    for x in -2..=2 {
        for y in -2..=2 {
            let z = rng.random::<f64>();
            scene.add(make_cube(x as f64, y as f64, z));
        }
    }

    let width = 1024.0;
    let height = 1024.0;

    let paths = scene
        .render(Vector::new(6.0, 5.0, 3.0))
        .width(width)
        .height(height)
        .call();
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

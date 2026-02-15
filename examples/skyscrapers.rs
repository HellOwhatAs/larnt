use larnt::{Cube, Scene, Vector};
use rand::{Rng, SeedableRng, rngs::SmallRng};

fn main() {
    let mut rng = SmallRng::seed_from_u64(42);
    let mut scene = Scene::new();
    let n = 860;

    for x in -n..=n {
        for y in -n..=n {
            let p = rng.random::<f64>() * 0.25 + 0.2;
            let fx = x as f64;
            let fy = y as f64;
            let fz = rng.random::<f64>() * 3.0 + 1.0;

            let shape = Cube::builder(
                Vector::new(fx - p, fy - p, 0.0),
                Vector::new(fx + p, fy + p, fz),
            )
            .build();
            scene.add(shape);
        }
    }

    let eye = Vector::new(13.75, 6.25, 18.0);
    let center = Vector::new(-8.0, -10.0, 0.0);
    let width = 2560.0;
    let height = 2560.0;

    let paths = scene
        .render(eye)
        .center(center)
        .width(width)
        .height(height)
        .fovy(65.0)
        .far(1e3)
        .call();
    paths
        .to_image(width, height, 1.5)
        .save("out.png")
        .expect("Failed to save PNG");
    paths.write_to_svg("out.svg", width, height).unwrap();
}

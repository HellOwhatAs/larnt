use larnt::{Scene, Sphere, SphereTexture::RandomCircles, Vector};
use rand::{Rng, SeedableRng, rngs::SmallRng};

fn main() {
    let mut scene = Scene::new();
    let mut rng = SmallRng::seed_from_u64(42);
    let n = 8;

    for x in -n..=n {
        for y in -n..=n {
            scene.add(
                Sphere::new(Vector::new(x as f64, y as f64, 0.0), 0.45)
                    .with_texture(RandomCircles(rng.random())),
            );
        }
    }

    let eye = Vector::new(8.0, 8.0, 1.0);
    let center = Vector::new(0.0, 0.0, -4.25);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene.render(eye, center, up, width, height, 50.0, 0.1, 100.0, 0.01);
    paths
        .to_image(width, height, 2.0)
        .save("out.png")
        .expect("Failed to save PNG");
    paths.write_to_svg("out.svg", width, height).unwrap();
}

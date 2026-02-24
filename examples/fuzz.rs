use larnt::{Scene, Sphere, SphereTexture, Vector};
use rand::{Rng, SeedableRng, rngs::SmallRng};

fn main() {
    let mut rng = SmallRng::seed_from_u64(42);

    let mut scene = Scene::new();
    let n = 10;

    for x in -n..=n {
        for y in -n..=n {
            let z = rng.random::<f64>() * 3.0;
            let v = Vector::new(x as f64, y as f64, z);
            let sphere = Sphere::builder(v, 0.45)
                .texture(SphereTexture::random_fuzz(42).call())
                .build();
            scene.add(sphere);
        }
    }

    let eye = Vector::new(8.0, 8.0, 8.0);
    let (width, height) = (1920.0, 1200.0);

    let paths = scene.render(eye).width(width).height(height).call();
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

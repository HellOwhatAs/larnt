use larnt::{ParametricSurface, Scene, Vector};

fn main() {
    let func = |u: f64, v: f64| -> Vector {
        let z = 2.0 * (u.powi(2) + v.powi(2)).sqrt().sin();
        Vector::new(u, v, z)
    };
    let mesh = ParametricSurface::new(func, (-20.0, 20.0), (-20.0, 20.0), 100, 100);
    let mut scene = Scene::new();
    scene.add(mesh);
    scene
        .render(Vector::new(45., 25., 30.))
        .center(Vector::new(4., 0., 0.))
        .call()
        .to_image(1024.0, 1024.0)
        .linewidth(1.5)
        .call()
        .save("out.png")
        .expect("Failed to save image");
}

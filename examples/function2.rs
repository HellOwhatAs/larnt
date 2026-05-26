use larnt::{ParametricSurface, Vector, render};

fn main() {
    let func = ParametricSurface::new(
        |x, y| Vector::new(x, y, (x).sin() * (y).cos() - (x.powi(2) + y.powi(2)) * 0.01),
        (-25.0, 25.0),
        (-25.0, 25.0),
        400,
        400,
    );

    let a = std::f64::consts::PI / 4.0;
    let eye = Vector::new(a.cos() * 28.0, a.sin() * 28.0, 10.0);
    let center = Vector::new(a.cos() * 9.0, a.sin() * 9.0, -4.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = render(vec![func])
        .eye(eye)
        .center(center)
        .width(width)
        .height(height)
        .fovy(70.0)
        .call();
    paths
        .to_image(width, height)
        .linewidth(0.8)
        .call()
        .save("out.png")
        .expect("Failed to save image");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

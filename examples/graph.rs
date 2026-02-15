use image::{Delay, DynamicImage, Frame, ImageBuffer, Rgb, codecs::gif::GifEncoder};
use larnt::{Scene, Sphere, Vector, new_transformed_cylinder, radians};
use std::{fs::File, sync::Arc, time::Duration};

fn save_gif_from_iter(
    frames_iter: impl Iterator<Item = ImageBuffer<Rgb<u8>, Vec<u8>>>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let mut encoder = GifEncoder::new(file);

    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    let gif_frames = frames_iter.map(|rgb_img| {
        let rgba_img = DynamicImage::ImageRgb8(rgb_img).into_rgba8();
        let delay = Delay::from_saturating_duration(Duration::from_millis(50));

        Frame::from_parts(rgba_img, 0, 0, delay)
    });

    encoder.encode_frames(gif_frames)?;
    Ok(())
}

fn render(frame: i32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let cx = radians(frame as f64).cos();
    let cy = radians(frame as f64).sin();
    let mut scene = Scene::new();
    let eye = Vector::new(cx, cy, 0.0).mul_scalar(8.0);

    let nodes = vec![
        Vector::new(1.047, -0.000, -1.312),
        Vector::new(-0.208, -0.000, -1.790),
        Vector::new(2.176, 0.000, -2.246),
        Vector::new(1.285, -0.001, 0.016),
        Vector::new(-1.276, -0.000, -0.971),
        Vector::new(-0.384, 0.000, -2.993),
        Vector::new(-2.629, -0.000, -1.533),
        Vector::new(-1.098, -0.000, 0.402),
        Vector::new(0.193, 0.005, 0.911),
        Vector::new(-1.934, -0.000, 1.444),
        Vector::new(2.428, -0.000, 0.437),
        Vector::new(0.068, -0.000, 2.286),
        Vector::new(-1.251, -0.000, 2.560),
        Vector::new(1.161, -0.000, 3.261),
        Vector::new(1.800, 0.001, -3.269),
        Vector::new(2.783, 0.890, -2.082),
        Vector::new(2.783, -0.889, -2.083),
        Vector::new(-2.570, -0.000, -2.622),
        Vector::new(-3.162, -0.890, -1.198),
        Vector::new(-3.162, 0.889, -1.198),
        Vector::new(-1.679, 0.000, 3.552),
        Vector::new(1.432, -1.028, 3.503),
        Vector::new(2.024, 0.513, 2.839),
        Vector::new(0.839, 0.513, 4.167),
    ];

    let edges: Vec<(usize, usize)> = vec![
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 4),
        (1, 5),
        (2, 14),
        (2, 15),
        (2, 16),
        (3, 8),
        (3, 10),
        (4, 6),
        (4, 7),
        (6, 17),
        (6, 18),
        (6, 19),
        (7, 8),
        (7, 9),
        (8, 11),
        (9, 12),
        (11, 12),
        (11, 13),
        (12, 20),
        (13, 21),
        (13, 22),
        (13, 23),
    ];

    // Add nodes as spheres
    for v in &nodes {
        scene.add(Sphere::builder(*v, 0.333).build());
    }

    // Add edges as cylinders
    for (i, j) in &edges {
        let v0 = nodes[*i];
        let v1 = nodes[*j];
        let cylinder = new_transformed_cylinder(v0, v1, 0.1).call();
        scene.add_arc(Arc::new(cylinder));
    }

    let (width, height) = (750.0, 750.0);
    let paths = scene
        .render(eye)
        .width(width)
        .height(height)
        .fovy(60.0)
        .call();
    paths.to_image(width, height, 2.5)
}

fn main() {
    let image_iter = (0..360).step_by(3).map(|i| render(i));
    save_gif_from_iter(image_iter, "output.gif").unwrap();
}

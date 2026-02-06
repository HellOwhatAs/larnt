use std::{fs::File, time::Duration};

use image::{codecs::gif::GifEncoder, Delay, DynamicImage, Frame, ImageBuffer, Rgb};
use larnt::{load_obj, Box as BBox, Matrix, Plane, Vector};

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

fn main() {
    let mut mesh = load_obj("examples/suzanne.obj").expect("Failed to load OBJ");
    mesh.fit_inside(
        BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)),
        Vector::new(0.5, 0.5, 0.5),
    );
    let slices = 128;
    let size = 1024.0;
    save_gif_from_iter(
        (0..slices).map(|i| {
            let p = (i as f64 / (slices - 1) as f64) * 2.0 - 1.0;
            let point = Vector::new(0.0, 0.0, p);
            let plane = Plane::new(point, Vector::new(0.0, 0.0, 1.0));
            let paths = plane.intersect_mesh(&mesh);
            let transform = Matrix::scale(Vector::new(size / 2.0, size / 2.0, 1.0))
                .translated(Vector::new(size / 2.0, size / 2.0, 0.0));
            let paths = paths.transform(&transform);
            paths.to_image(size, size, 2.5)
        }),
        "output.gif",
    )
    .expect("Failed to save GIF");
}

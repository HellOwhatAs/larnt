use image::codecs::gif::GifEncoder;
use image::{Delay, Frame, ImageBuffer, Rgba};
use larnt::{
    CubeTexture, Cylinder, Matrix, Scene, Shape, Sphere, TransformedShape, Vector, new_difference,
    new_intersection, radians,
};
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

fn save_gif_from_iter(
    frames_iter: impl Iterator<Item = ImageBuffer<Rgba<u8>, Vec<u8>>>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let mut encoder = GifEncoder::new(file);

    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    let gif_frames = frames_iter.map(|rgba_img| {
        let delay = Delay::from_saturating_duration(Duration::from_millis(50));
        Frame::from_parts(rgba_img, 0, 0, delay)
    });

    encoder.encode_frames(gif_frames)?;
    Ok(())
}

fn main() {
    let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(
        Sphere::builder(Vector::default(), 1.0)
            .texture(larnt::SphereTexture::lat_lng().call())
            .build(),
    );
    let cube: Arc<dyn Shape + Send + Sync> = Arc::new(
        larnt::Cube::builder(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8))
            .texture(CubeTexture::striped().stripes(40).call())
            .build(),
    );

    let cyl1: Arc<dyn Shape + Send + Sync> = Arc::new(
        Cylinder::builder(0.4, -2.0, 2.0)
            .texture(larnt::CylinderTexture::striped().call())
            .build(),
    );
    let cyl2: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
        Arc::new(
            Cylinder::builder(0.4, -2.0, 2.0)
                .texture(larnt::CylinderTexture::striped().call())
                .build(),
        ),
        Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
    ));
    let cyl3: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
        Arc::new(
            Cylinder::builder(0.4, -2.0, 2.0)
                .texture(larnt::CylinderTexture::striped().call())
                .build(),
        ),
        Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
    ));

    let shape = new_difference(vec![new_intersection(vec![sphere, cube]), cyl1, cyl2, cyl3]);

    let image_iter = (0..90).step_by(2).map(|i| {
        let mut scene = Scene::new();
        let m = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(i as f64));
        scene.add_arc(Arc::new(TransformedShape::new(Arc::clone(&shape), m)));

        let eye = Vector::new(0.0, 6.0, 2.0);
        let width = 750.0;
        let height = 750.0;

        let paths = scene
            .render(eye)
            .width(width)
            .height(height)
            .fovy(20.0)
            .call();
        paths.to_image(width, height).linewidth(2.5).call()
    });

    save_gif_from_iter(image_iter, "output.gif").unwrap();
}

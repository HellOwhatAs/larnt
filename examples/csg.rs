use image::codecs::gif::GifEncoder;
use image::{Delay, Frame, ImageBuffer, Rgba};
use larnt::{
    Cube, CubeTexture, Cylinder, Matrix, Primitive, Sphere, TransformedShape, Vector,
    new_difference, new_intersection, radians, render,
};
use std::fs::File;
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
    let sphere = Sphere::builder(Vector::default(), 1.0)
        .texture(larnt::SphereTexture::lat_lng().call())
        .build();
    let cube = Cube::builder(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8))
        .texture(CubeTexture::striped().stripes(40).call())
        .build();

    let cyl1 = Cylinder::builder(0.4, -2.0, 2.0)
        .texture(larnt::CylinderTexture::striped().call())
        .build();
    let cyl2: TransformedShape<Primitive> = TransformedShape::new(
        Cylinder::builder(0.4, -2.0, 2.0)
            .texture(larnt::CylinderTexture::striped().call())
            .build()
            .into(),
        Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
    );
    let cyl3: TransformedShape<Primitive> = TransformedShape::new(
        Cylinder::builder(0.4, -2.0, 2.0)
            .texture(larnt::CylinderTexture::striped().call())
            .build()
            .into(),
        Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
    );

    let shape: &Primitive = &new_difference(vec![
        new_intersection(vec![sphere.into(), cube.into()]),
        cyl1.into(),
        cyl2.into(),
        cyl3.into(),
    ]);

    let image_iter = (0..90).step_by(2).map(|i| {
        let mut shapes = Vec::new();
        let m = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(i as f64));
        shapes.push(TransformedShape::new(shape, m));

        let eye = Vector::new(0.0, 6.0, 2.0);
        let width = 750.0;
        let height = 750.0;

        let paths = render(shapes)
            .eye(eye)
            .width(width)
            .height(height)
            .fovy(20.0)
            .call();
        paths.to_image(width, height).linewidth(2.5).call()
    });

    save_gif_from_iter(image_iter, "output.gif").unwrap();
}

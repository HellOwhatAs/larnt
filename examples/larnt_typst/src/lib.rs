pub mod constructor;
pub mod interp;

use ciborium::de::from_reader;
use image::{ImageFormat, Rgba};
use wasm_minimal_protocol::*;

initiate_protocol!();

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Svg,
    Png { linewidth: f64 },
}

#[derive(serde::Deserialize)]
struct RenderArgs {
    eye: [f64; 3],
    center: [f64; 3],
    up: [f64; 3],
    width: f64,
    height: f64,
    fovy: f64,
    near: f64,
    far: f64,
    step: f64,
    format: Format,
}

#[wasm_func]
fn render(render_args: &[u8], shapes: &[u8]) -> Result<Vec<u8>, String> {
    let args: RenderArgs = from_reader(render_args).map_err(|e| e.to_string())?;
    let shapes: Vec<constructor::LnShape> = from_reader(shapes).map_err(|e| e.to_string())?;
    let paths = constructor::render(
        shapes.into_iter(),
        args.eye,
        args.center,
        args.up,
        args.width,
        args.height,
        args.fovy,
        args.near,
        args.far,
        args.step,
    )?;
    Ok(match args.format {
        Format::Svg => paths.to_svg(args.width, args.height).into_bytes(),
        Format::Png { linewidth } => {
            let image = paths
                .to_image(args.width, args.height)
                .linewidth(linewidth)
                .background(Rgba([255, 255, 255, 0]))
                .call();
            let mut cursor = std::io::Cursor::new(Vec::new());
            image
                .write_to(&mut cursor, ImageFormat::Png)
                .map_err(|e| e.to_string())?;
            cursor.into_inner()
        }
    })
}

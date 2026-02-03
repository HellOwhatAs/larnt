pub mod constructor;
pub mod interp;

use ciborium::de::from_reader;
use wasm_minimal_protocol::*;

initiate_protocol!();

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
}

#[wasm_func]
fn render(render_args: &[u8], shapes: &[u8]) -> Result<Vec<u8>, String> {
    let args: RenderArgs = from_reader(render_args).map_err(|e| e.to_string())?;
    let shapes: Vec<constructor::LnShape> = from_reader(shapes).map_err(|e| e.to_string())?;

    Ok(constructor::render(
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
    )?
    .to_svg(args.width, args.height)
    .into_bytes())
}

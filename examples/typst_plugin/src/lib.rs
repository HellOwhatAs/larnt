use ln::{Cube, Scene, Vector};
use wasm_minimal_protocol::*;
// Only necessary when using cbor for passing arguments.
use ciborium::de::from_reader;

initiate_protocol!();

#[wasm_func]
fn example0() -> Vec<u8> {
    let mut scene = Scene::new();
    scene.add(Cube::new(
        Vector::new(-1.0, -1.0, -1.0),
        Vector::new(1.0, 1.0, 1.0),
    ));

    // define camera parameters
    let eye = Vector::new(4.0, 3.0, 2.0); // camera position
    let center = Vector::new(0.0, 0.0, 0.0); // camera looks at
    let up = Vector::new(0.0, 0.0, 1.0); // up direction

    // define rendering parameters
    let width = 1024.0; // rendered width
    let height = 1024.0; // rendered height
    let fovy = 50.0; // vertical field of view, degrees
    let znear = 0.1; // near z plane
    let zfar = 10.0; // far z plane
    let step = 0.01; // how finely to chop the paths for visibility testing

    // compute 2D paths that depict the 3D scene
    let paths = scene.render(eye, center, up, width, height, fovy, znear, zfar, step);

    // save the result as an svg
    // paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
    paths.to_svg(width, height).into_bytes()
}

#[wasm_func]
fn skyscrapers() -> Vec<u8> {
    use rand::{Rng, SeedableRng, rngs::SmallRng};
    let mut rng = SmallRng::seed_from_u64(42);
    let mut scene = Scene::new();
    let n = 15;

    for x in -n..=n {
        for y in -n..=n {
            let p = rng.r#gen::<f64>() * 0.25 + 0.2;
            let fx = x as f64;
            let fy = y as f64;
            let fz = rng.r#gen::<f64>() * 3.0 + 1.0;

            // Skip one building to create a gap (matching original example)
            if x == 2 && y == 1 {
                continue;
            }

            let shape = Cube::new(
                Vector::new(fx - p, fy - p, 0.0),
                Vector::new(fx + p, fy + p, fz),
            );
            scene.add(shape);
        }
    }

    let eye = Vector::new(1.75, 1.25, 6.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = scene.render(eye, center, up, width, height, 100.0, 0.1, 100.0, 0.01);
    paths.to_svg(width, height).into_bytes()
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
}

#[wasm_func]
fn render(render_args: &[u8], items: &[u8]) -> Result<Vec<u8>, String> {
    let args: RenderArgs = from_reader(render_args).map_err(|e| e.to_string())?;
    let mut scene = Scene::new();

    let shapes: Vec<[[f64; 3]; 2]> = from_reader(items).map_err(|e| e.to_string())?;
    for shape in shapes {
        let min = Vector::new(shape[0][0], shape[0][1], shape[0][2]);
        let max = Vector::new(shape[1][0], shape[1][1], shape[1][2]);
        let cube = Cube::new(min, max);
        scene.add(cube);
    }

    let paths = scene.render(
        Vector::new(args.eye[0], args.eye[1], args.eye[2]),
        Vector::new(args.center[0], args.center[1], args.center[2]),
        Vector::new(args.up[0], args.up[1], args.up[2]),
        args.width,
        args.height,
        args.fovy,
        args.near,
        args.far,
        args.step,
    );
    Ok(paths.to_svg(args.width, args.height).into_bytes())
}

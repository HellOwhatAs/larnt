use ln::{load_obj, Matrix, Plane, Vector, Box as BBox};

const SLICES: usize = 32;
const SIZE: f64 = 1024.0;

fn main() {
    let mut mesh = load_obj("examples/suzanne.obj").expect("Failed to load OBJ");
    mesh.fit_inside(
        BBox::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)),
        Vector::new(0.5, 0.5, 0.5),
    );
    
    for i in 0..SLICES {
        println!("slice{:04}", i);
        let p = (i as f64 / (SLICES - 1) as f64) * 2.0 - 1.0;
        let point = Vector::new(0.0, 0.0, p);
        let plane = Plane::new(point, Vector::new(0.0, 0.0, 1.0));
        let paths = plane.intersect_mesh(&mesh);
        let transform = Matrix::scale(Vector::new(SIZE / 2.0, SIZE / 2.0, 1.0))
            .translated(Vector::new(SIZE / 2.0, SIZE / 2.0, 0.0));
        let paths = paths.transform(&transform);
        paths.write_to_png(&format!("slice{:04}.png", i), SIZE, SIZE);
    }
}

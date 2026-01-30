use ln::{new_transformed_outline_cylinder, OutlineSphere, radians, Scene, Vector};
use std::sync::Arc;

fn render(frame: i32) {
    let cx = radians(frame as f64).cos();
    let cy = radians(frame as f64).sin();
    let mut scene = Scene::new();
    let eye = Vector::new(cx, cy, 0.0).mul_scalar(8.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);

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
        scene.add(OutlineSphere::new(eye, up, *v, 0.333));
    }

    // Add edges as cylinders
    for (i, j) in &edges {
        let v0 = nodes[*i];
        let v1 = nodes[*j];
        let cylinder = new_transformed_outline_cylinder(eye, up, v0, v1, 0.1);
        scene.add_arc(Arc::new(cylinder));
    }

    let width = 750.0;
    let height = 750.0;
    let paths = scene.render(eye, center, up, width, height, 60.0, 0.1, 100.0, 0.01);
    paths.write_to_png(&format!("out{:03}.png", frame), width, height);
}

fn main() {
    for i in (0..360).step_by(2) {
        render(i);
    }
}

use larnt::{BBox, Matrix, Mesh, TransformedShape, Vector, load_obj, render};

fn main() {
    let mesh: Mesh = load_obj("examples/suzanne.obj").expect("Failed to load OBJ");
    let matrix = mesh
        .fit_inside(
            BBox::new(Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 1.0)),
            Vector::new(0.0, 0.0, 0.0),
        )
        .translated(Vector::new(-0.5, -0.5, -0.5));
    let mesh = TransformedShape::new(mesh, matrix);

    let transform = Matrix::rotate(Vector::new(0.0, 1.0, 0.0), 0.5);

    let eye = Vector::new(-0.5, 0.5, 2.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let width = 1024.0;
    let height = 1024.0;

    let paths = render(vec![TransformedShape::new(mesh, transform)])
        .eye(eye)
        .up(up)
        .width(width)
        .height(height)
        .fovy(35.0)
        .call();
    paths.write_to_png("out.png", width, height);
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

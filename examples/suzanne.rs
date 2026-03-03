use larnt::{BBox, Mesh, TransformedShape, Vector, load_obj, render};

fn main() {
    let mut mesh: Mesh = load_obj("examples/suzanne.obj").expect("Failed to load OBJ");
    mesh.texture = larnt::mesh::MeshTexture::silhouette().cos_theta(0.5).call();
    let matrix = mesh
        .fit_inside(
            BBox::new(Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 1.0)),
            Vector::new(0.0, 0.0, 0.0),
        )
        .translated(Vector::new(-0.5, -0.5, -0.5))
        .rotated(Vector::new(0.0, 1.0, 0.0), 0.5);

    let eye = Vector::new(-0.5, 0.5, 2.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let width = 1024.0;
    let height = 1024.0;
    let paths = render(vec![TransformedShape::new(mesh, matrix)])
        .eye(eye)
        .up(up)
        .width(width)
        .height(height)
        .fovy(35.0)
        .call();
    paths
        .write_to_png("out.png", width, height)
        .expect("Failed to write PNG");
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

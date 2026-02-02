use ln::{
    new_transformed_cone, new_transformed_cylinder, new_transformed_outline_cone,
    new_transformed_outline_cylinder, Cube, CubeTexture, OutlineSphere, Scene, Sphere,
    SphereTexture, Vector,
};

fn main() {
    // create a scene and add a single cube
    let mut scene = Scene::new();

    // define camera parameters
    let eye = Vector::new(2.0, 7.0, 5.0); // camera position
    let center = Vector::new(1.5, 2.0, 0.0); // camera looks at
    let up = Vector::new(0.0, 0.0, 1.0); // up direction

    // define rendering parameters
    let width = 1024.0; // rendered width
    let height = 1024.0; // rendered height
    let fovy = 50.0; // vertical field of view, degrees
    let znear = 0.1; // near z plane
    let zfar = 10.0; // far z plane
    let step = 0.01; // how finely to chop the paths for visibility testing

    scene.add(Cube::new(
        Vector::new(0.0, 0.0, 0.0),
        Vector::new(1.0, 1.0, 1.0),
    ));
    scene.add(
        Cube::new(Vector::new(1.5, 0.0, 0.0), Vector::new(2.5, 1.0, 1.0))
            .with_texture(CubeTexture::Striped(8)),
    );
    scene.add(Sphere::new(Vector::new(0.5, 2.0, 0.5), 0.5));
    scene.add(
        Sphere::new(Vector::new(2.0, 2.0, 0.5), 0.5).with_texture(SphereTexture::RandomCircles(42)),
    );
    scene.add(
        Sphere::new(Vector::new(0.5, 3.5, 0.5), 0.5)
            .with_texture(SphereTexture::RandomEquators(42)),
    );
    scene.add(OutlineSphere::new(eye, up, Vector::new(2.0, 3.5, 0.5), 0.5));
    scene.add(new_transformed_cone(
        up,
        Vector::new(-1.0, 0.5, 0.0),
        Vector::new(-1.0, 0.5, 1.0),
        0.5,
    ));
    scene.add(new_transformed_outline_cone(
        eye,
        up,
        Vector::new(-1.0, 2.0, 0.0),
        Vector::new(-1.0, 2.0, 1.0),
        0.5,
    ));
    scene.add(new_transformed_cylinder(
        up,
        Vector::new(3.5, 0.5, 0.0),
        Vector::new(3.5, 0.5, 1.0),
        0.5,
    ));
    scene.add(new_transformed_outline_cylinder(
        eye,
        up,
        Vector::new(3.5, 2.0, 0.0),
        Vector::new(3.5, 2.0, 1.0),
        0.5,
    ));

    // compute 2D paths that depict the 3D scene
    let paths = scene.render(eye, center, up, width, height, fovy, znear, zfar, step);

    // save the result as a png
    paths.write_to_png("out.png", width, height);

    // save the result as an svg
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

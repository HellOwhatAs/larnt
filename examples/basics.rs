use larnt::{
    ConeTexture, Cube, CubeTexture, CylinderTexture, Scene, Sphere, SphereTexture, Vector,
    new_transformed_cone, new_transformed_cylinder,
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
    let step = 1.0; // how finely to chop the paths for visibility testing

    scene.add(Cube::builder(Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 1.0)).build());
    scene.add(
        Cube::builder(Vector::new(1.5, 0.0, 0.0), Vector::new(2.5, 1.0, 1.0))
            .texture(CubeTexture::striped().call())
            .build(),
    );
    scene.add(
        Sphere::builder(Vector::new(0.5, 2.0, 0.5), 0.5)
            .texture(SphereTexture::lat_lng().call())
            .build(),
    );
    scene.add(
        Sphere::builder(Vector::new(2.0, 2.0, 0.5), 0.5)
            .texture(SphereTexture::random_circles(42).call())
            .build(),
    );
    scene.add(
        Sphere::builder(Vector::new(0.5, 3.5, 0.5), 0.5)
            .texture(SphereTexture::random_equators(42).call())
            .build(),
    );
    scene.add(Sphere::builder(Vector::new(2.0, 3.5, 0.5), 0.5).build());
    scene.add(
        new_transformed_cone(
            Vector::new(-1.0, 0.5, 0.0),
            Vector::new(-1.0, 0.5, 1.0),
            0.5,
        )
        .texture(ConeTexture::Striped(12))
        .call(),
    );
    scene.add(
        new_transformed_cone(
            Vector::new(-1.0, 2.0, 0.0),
            Vector::new(-1.0, 2.0, 1.0),
            0.5,
        )
        .call(),
    );
    scene.add(
        new_transformed_cylinder(Vector::new(3.5, 0.5, 0.0), Vector::new(3.5, 0.5, 1.0), 0.5)
            .texture(CylinderTexture::Striped(36))
            .call(),
    );
    scene.add(
        new_transformed_cylinder(Vector::new(3.5, 2.0, 0.0), Vector::new(3.5, 2.0, 1.0), 0.5)
            .call(),
    );

    // compute 2D paths that depict the 3D scene
    let paths = scene
        .render(eye)
        .center(center)
        .up(up)
        .width(width)
        .height(height)
        .fovy(fovy)
        .near(znear)
        .far(zfar)
        .step(step)
        .call();

    // save the result as a png
    paths.write_to_png("out.png", width, height);

    // save the result as an svg
    paths
        .write_to_svg("out.svg", width, height)
        .expect("Failed to write SVG");
}

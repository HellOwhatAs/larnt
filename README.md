# `larnt` The 3D Line Art Engine

[![crates.io](https://img.shields.io/crates/v/larnt)](https://crates.io/crates/larnt)
[![Typst Universe](https://img.shields.io/badge/Typst-Universe-239dad)](https://typst.app/universe/package/larnt/)
[![Repo](https://img.shields.io/badge/GitHub-repo-444)](https://github.com/HellOwhatAs/larnt/)

`larnt` is a vector-based 3D renderer written in Rust. It is used to produce 2D vector graphics (think SVGs) depicting 3D scenes.

_The output of an OpenGL pipeline is a rastered image. The output of `larnt` is a set of 2D vector paths._

> This project is a Rust rewrite of the original [ln](https://github.com/fogleman/ln) by [Michael Fogleman](https://github.com/fogleman).

## Examples

<table>
    <tr>
        <td>
            <a href="examples/basics.rs">
                <img alt="basics" src="https://github.com/user-attachments/assets/eba5a931-1465-4c65-add2-31a17db63b18" />
            </a>
        </td>
        <td>
            <a href="examples/beads.rs">
                <img alt="beads" src="https://github.com/user-attachments/assets/bd96fa4c-4ef3-49e6-a572-2680e417fa05" />
            </a>
        </td>
        <td>
            <a href="examples/csg.rs">
                <img alt="csg" src="https://github.com/user-attachments/assets/5f70e7ca-ea2d-4804-8948-57b11251f4eb" />
            </a>
        </td>
    </tr>
    <tr>
        <td>
            <a href="examples/example1.rs">
                <img alt="example1" src="https://github.com/user-attachments/assets/a8b92a0c-4d07-41bd-8ce9-7fec12215b5d" />
            </a>
        </td>
        <td>
            <a href="examples/function2.rs">
                <img alt="function2" src="https://github.com/user-attachments/assets/d47697cc-ffb7-416a-ab2b-635e87160b1d" />
            </a>
        </td>
        <td>
            <a href="examples/graph.rs">
                <img alt="graph" src="https://github.com/user-attachments/assets/b9b072a8-c81f-45a5-a40d-33559bea932e" />
            </a>
        </td>
    </tr>
    <tr>
        <td>
            <a href="examples/skyscrapers.rs">
                <img alt="skyscrapers" src="https://github.com/user-attachments/assets/e1d8e39d-b670-4350-9d57-bcfafa73ed92" />
            </a>
        </td>
        <td>
            <a href="examples/suzanne.rs">
                <img alt="suzanne" src="https://github.com/user-attachments/assets/3288522d-61b4-4680-ba55-3d536e0e96ce" />
            </a>
        </td>
        <td>
            <a href="examples/test.rs">
                <img alt="test" src="https://github.com/user-attachments/assets/9abc50da-b1fe-4c23-bc46-588bd26e93d8" />
            </a>
        </td>
    </tr>
    <tr>
        <td>
            <a href="examples/torus.rs">
                <img alt="torus" src="https://github.com/user-attachments/assets/fb63c340-8288-4962-ad20-def83f4a9311" />
            </a>
        </td>
        <td>
            <a href="examples/mobius.rs">
                <img alt="mobius" src="https://github.com/user-attachments/assets/99ccff8f-6d5f-4178-ac2b-2201fee329f7" />
            </a>
        </td>
        <td>
            <a href="examples/klein_bottle.rs">
                <img alt="klein_bottle" src="https://github.com/user-attachments/assets/68214ff1-b2de-4532-ab05-62a04c355a93" />
            </a>
        </td>
    </tr>
</table>

_Click on the example image to jump to the code._

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
larnt = "0.1.0"
```

Or for the latest development version

```toml
[dependencies]
larnt = { git = "https://github.com/HellOwhatAs/larnt.git" }
```

## Features

- Primitives
  - Sphere
  - Cube
  - Triangle
  - Cylinder
  - Cone
  - 3D Surface
- Triangle Meshes
  - OBJ & STL
- Vector-based "Texturing"
- CSG (Constructive Solid Geometry) Operations
  - Intersection
  - Difference
- Output to PNG or SVG

## How it Works

To understand how `larnt` works, it's useful to start with the `Shape` trait:

```rust
use larnt::{BBox, Hit, Paths, Ray, Vector, RenderArgs};

pub trait Shape {
    fn bounding_box(&self) -> BBox;
    fn contains(&self, v: Vector, f: f64) -> bool;
    fn intersect(&self, r: Ray) -> Hit;
    fn paths(&self, args: &RenderArgs) -> Paths<Vector>;
}
```

Each shape must provide some `Paths` which are 3D polylines on the surface
of the solid. Ultimately anything drawn in the final image is based on these
paths. These paths can be anything. For a sphere they could be lat/lng grid
lines, a triangulated-looking surface, dots on the surface, etc. This is what
we call vector-based texturing. Each built-in `Shape` ships with a default
`paths()` function (e.g. a `Cube` simply draws the outline of a cube) but you
can easily provide your own.

Each shape must also provide an `intersect` method that lets the engine test
for ray-solid intersection. This is how the engine knows what is visible to the
camera and what is hidden.

All of the `Paths` are chopped up to some granularity and each point is tested
by shooting a ray toward the camera. If there is no intersection, that point is
visible. If there is an intersection, it is hidden and will not be rendered.

The visible points are then transformed into 2D space using transformation
matrices. The result can then be rendered as PNG or SVG.

The `contains` method is only needed for CSG (Constructive Solid Geometry)
operations.

## Hello World: A Single Cube

### The Code

```rust
use larnt::{Cube, Vector, render};

// create a scene and add a single cube
let mut shapes = Vec::new();
shapes.push(Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build());

let eye = Vector::new(4.0, 3.0, 2.0); // camera position
let width = 1024.0; // rendered width
let height = 1024.0; // rendered height

// compute 2D paths that depict the 3D scene
let paths = render(shapes)
    .eye(eye)
    .center(Vector::new(0.0, 0.0, 0.0)) // camera looks at
    .up(Vector::new(0.0, 0.0, 1.0)) // up direction
    .width(width) // rendered width
    .height(height) // rendered height
    .fovy(50.0) // vertical field of view, degrees
    .near(0.1) // near plane
    .far(1000.0) // far plane
    // how finely to chop the paths for visibility testing
    // unit is the same as the scene's units
    .step(1.0)
    .call();

// render the paths in an image
paths.write_to_png("out.png", width, height).expect("Failed to write PNG");

// save the paths as an svg
paths
    .write_to_svg("out.svg", width, height)
    .expect("Failed to write SVG");
```

### The Output

<img width="250px" alt="example0" src="https://github.com/user-attachments/assets/3ab195bf-0e3d-4304-b68d-e7dac0b501d9" />

## Custom Texturing

Suppose we want to draw cubes with vertical stripes on their sides, as
shown in the skyscrapers example above. We can implement the `Shape` trait
for a custom type.

```rust
use larnt::{BBox, Cube, Shape, RenderArgs, Paths, Vector, Hit, Ray};

struct StripedCube {
    cube: Cube,
    stripes: i32,
}

impl Shape for StripedCube {
    fn bounding_box(&self) -> BBox {
        self.cube.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.cube.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.cube.intersect(r)
    }

    fn paths(&self, _: &RenderArgs) -> Paths<Vector> {
        let mut paths = Paths::new();
        let (x1, y1, z1) = (self.cube.min.x, self.cube.min.y, self.cube.min.z);
        let (x2, y2, z2) = (self.cube.max.x, self.cube.max.y, self.cube.max.z);

        for i in 0..=self.stripes {
            let p = i as f64 / self.stripes as f64;
            let x = x1 + (x2 - x1) * p;
            let y = y1 + (y2 - y1) * p;
            paths.new_path().extend([Vector::new(x, y1, z1), Vector::new(x, y1, z2)]);
            paths.new_path().extend([Vector::new(x, y2, z1), Vector::new(x, y2, z2)]);
            paths.new_path().extend([Vector::new(x1, y, z1), Vector::new(x1, y, z2)]);
            paths.new_path().extend([Vector::new(x2, y, z1), Vector::new(x2, y, z2)]);
        }
        paths
    }
}
```

Now `StripedCube` instances can be added to the scene.

## Constructive Solid Geometry (CSG)

You can easily construct complex solids using Intersection, Difference.

```rust
use larnt::{
    Cube, CubeTexture, Cylinder, Matrix, Primitive, Sphere, SphereTexture, TransformedShape,
    Vector, new_difference, new_intersection, radians,
};

let shape: Primitive = new_difference(vec![
    new_intersection(vec![
        Sphere::builder(Vector::default(), 1.0)
            .texture(SphereTexture::lat_lng().call())
            .build()
            .into(),
        Cube::builder(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8))
            .texture(CubeTexture::striped().stripes(10).call())
            .build()
            .into(),
    ]),
    Cylinder::builder(0.4, -2.0, 2.0).build().into(),
    TransformedShape::new(
        Cylinder::builder(0.4, -2.0, 2.0).build().into(),
        Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
    ).into(),
    TransformedShape::new(
        Cylinder::builder(0.4, -2.0, 2.0).build().into(),
        Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
    ).into(),
]);
```

This is `(Sphere & Cube) - (Cylinder | Cylinder | Cylinder)`.

Unfortunately, it's difficult to compute the joint formed at the boundaries of these combined shapes, so sufficient texturing is needed on the original solids for a decent result.

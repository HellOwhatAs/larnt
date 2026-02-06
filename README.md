# `larnt` The 3D Line Art Engine

[![crates.io](https://img.shields.io/crates/v/larnt)](https://crates.io/crates/larnt)
[![Typst Universe](https://img.shields.io/badge/Typst-Universe-239dad)](https://typst.app/universe/package/larnt/)
[![Repo](https://img.shields.io/badge/GitHub-repo-444)](https://github.com/HellOwhatAs/larnt/tree/master/examples/larnt_typst)

`larnt` is a vector-based 3D renderer written in Rust. It is used to produce 2D vector graphics (think SVGs) depicting 3D scenes.

*The output of an OpenGL pipeline is a rastered image. The output of `larnt` is a set of 2D vector paths.*

> This project is a Rust rewrite of the original [Go implementation](https://github.com/fogleman/ln) by [Michael Fogleman](https://github.com/fogleman).

## Examples
<table>
    <tr>
        <td>
            <a href="examples/basics.rs">
                <img  alt="basics"
                    src="https://github.com/user-attachments/assets/d989173e-2e17-4e51-b40d-1d90fece2c7e" />
            </a>
        </td>
        <td>
            <a href="examples/beads.rs">
                <img  alt="beads"
                    src="https://github.com/user-attachments/assets/4dec924e-84a5-4e68-963c-f91bbf23b527" />
            </a>
        </td>
        <td>
            <a href="examples/csg.rs">
                <img  alt="csg"
                    src="https://github.com/user-attachments/assets/dbc90e39-4373-4de1-acd4-55fc07220ebf" />
            </a>
        </td>
    </tr>
    <tr>
        <td>
            <a href="examples/example1.rs">
                <img  alt="example1"
                    src="https://github.com/user-attachments/assets/49cbdc1f-55b4-41ee-b346-43372b74abc9" />
            </a>
        </td>
        <td>
            <a href="examples/function2.rs">
                <img  alt="function2"
                    src="https://github.com/user-attachments/assets/5353c28c-33a6-4a14-a6b0-5fcd85dcb932" />
            </a>
        </td>
        <td>
            <a href="examples/graph.rs">
                <img  alt="graph"
                    src="https://github.com/user-attachments/assets/e0b3d6bc-e656-4e3c-bf52-cf4c8cadb78e" />
            </a>
        </td>
    </tr>
    <tr>
        <td>
            <a href="examples/skyscrapers.rs">
                <img  alt="skyscrapers"
                    src="https://github.com/user-attachments/assets/35e9d8f2-80fa-4da3-a5bd-8d5d81f832f2" />
            </a>
        </td>
        <td>
            <a href="examples/suzanne.rs">
                <img  alt="suzanne"
                    src="https://github.com/user-attachments/assets/f5971deb-94dc-4688-beeb-43cf463aab64" />
            </a>
        </td>
        <td>
            <a href="examples/test.rs">
                <img  alt="test"
                    src="https://github.com/user-attachments/assets/a4031cad-9a6f-4a0a-bfcf-87a5b49f6a4a" />
        </td>
    </tr>
</table>

*Click on the example image to jump to the code.*

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
larnt = "0.1.0"
```

## Features

- Primitives
	- Sphere
	- Cube
	- Triangle
	- Cylinder
	- Cone
	- 3D Functions
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
use larnt::{Box, Hit, Paths, Ray, Vector};

pub trait Shape {
    fn compile(&mut self) {}
    fn bounding_box(&self) -> Box;
    fn contains(&self, v: Vector, f: f64) -> bool;
    fn intersect(&self, r: Ray) -> Hit;
    fn paths(&self) -> Paths;
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
use larnt::{Cube, Scene, Vector};

fn main() {
    // create a scene and add a single cube
    let mut scene = Scene::new();
    scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));

    // define camera parameters
    let eye = Vector::new(4.0, 3.0, 2.0);    // camera position
    let center = Vector::new(0.0, 0.0, 0.0); // camera looks at
    let up = Vector::new(0.0, 0.0, 1.0);     // up direction

    // define rendering parameters
    let width = 1024.0;  // rendered width
    let height = 1024.0; // rendered height
    let fovy = 50.0;     // vertical field of view, degrees
    let znear = 0.1;     // near z plane
    let zfar = 10.0;     // far z plane
    let step = 0.01;     // how finely to chop the paths for visibility testing

    // compute 2D paths that depict the 3D scene
    let paths = scene.render(eye, center, up, width, height, fovy, znear, zfar, step);

    // render the paths in an image
    paths.write_to_png("out.png", width, height);

    // save the paths as an svg
    paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
}
```

### The Output
<img width="250px" alt="example0" src="https://github.com/user-attachments/assets/3ab195bf-0e3d-4304-b68d-e7dac0b501d9" />

## Custom Texturing

Suppose we want to draw cubes with vertical stripes on their sides, as
shown in the skyscrapers example above. We can implement the `Shape` trait
for a custom type.

```rust
use larnt::{Cube, Shape, Paths, Vector, Box, Hit, Ray};

struct StripedCube {
    cube: Cube,
    stripes: i32,
}

impl Shape for StripedCube {
    fn bounding_box(&self) -> Box {
        self.cube.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.cube.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.cube.intersect(r)
    }

    fn paths(&self) -> Paths {
        let mut paths = Vec::new();
        let (x1, y1, z1) = (self.cube.min.x, self.cube.min.y, self.cube.min.z);
        let (x2, y2, z2) = (self.cube.max.x, self.cube.max.y, self.cube.max.z);
        
        for i in 0..=self.stripes {
            let p = i as f64 / self.stripes as f64;
            let x = x1 + (x2 - x1) * p;
            let y = y1 + (y2 - y1) * p;
            paths.push(vec![Vector::new(x, y1, z1), Vector::new(x, y1, z2)]);
            paths.push(vec![Vector::new(x, y2, z1), Vector::new(x, y2, z2)]);
            paths.push(vec![Vector::new(x1, y, z1), Vector::new(x1, y, z2)]);
            paths.push(vec![Vector::new(x2, y, z1), Vector::new(x2, y, z2)]);
        }
        Paths::from_vec(paths)
    }
}
```

Now `StripedCube` instances can be added to the scene.

## Constructive Solid Geometry (CSG)

You can easily construct complex solids using Intersection, Difference.

```rust
use larnt::{new_difference, new_intersection, radians, Cube, Cylinder, Matrix, Sphere, TransformedShape, Vector};
use std::sync::Arc;

let shape = new_difference(vec![
    new_intersection(vec![
        Arc::new(Sphere::new(Vector::default(), 1.0)),
        Arc::new(Cube::new(Vector::new(-0.8, -0.8, -0.8), Vector::new(0.8, 0.8, 0.8))),
    ]),
    Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
    Arc::new(TransformedShape::new(
        Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
        Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
    )),
    Arc::new(TransformedShape::new(
        Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
        Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
    )),
]);
```

This is `(Sphere & Cube) - (Cylinder | Cylinder | Cylinder)`.

Unfortunately, it's difficult to compute the joint formed at the boundaries of these combined shapes, so sufficient texturing is needed on the original solids for a decent result.

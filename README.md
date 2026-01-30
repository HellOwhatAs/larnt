# `ln` The 3D Line Art Engine

`ln` is a vector-based 3D renderer written in Rust. It is used to produce 2D
vector graphics (think SVGs) depicting 3D scenes.

*The output of an OpenGL pipeline is a rastered image. The output of `ln` is
a set of 2D vector paths.*

![Examples](http://i.imgur.com/HY2Fg2t.png)

## Motivation

I created this so I could plot 3D drawings with my
[Makeblock XY Plotter](http://www.makeblock.cc/xy-plotter-robot-kit/).

Here's one of my drawings from the plotter...

![Example](http://i.imgur.com/NbgpUhQ.jpg)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ln = { git = "https://github.com/fogleman/ln" }
```

Or clone and build locally:

```bash
git clone https://github.com/fogleman/ln
cd ln
cargo build --release
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

To understand how `ln` works, it's useful to start with the `Shape` trait:

```rust
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
use ln::{Cube, Scene, Vector};

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

![Cube](http://i.imgur.com/d2dGrOJ.png)

## Custom Texturing

Suppose we want to draw cubes with vertical stripes on their sides, as
shown in the skyscrapers example above. We can implement the `Shape` trait
for a custom type.

```rust
use ln::{Cube, Shape, Paths, Vector, Box, Hit, Ray};

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
use ln::{new_difference, new_intersection, radians, Cylinder, Matrix, Sphere, TransformedShape, Vector};
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

![Example](http://i.imgur.com/gk8UtVK.gif)

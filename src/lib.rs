//! # ln - The 3D Line Art Engine
//!
//! `ln` is a vector-based 3D renderer written in Rust. It is used to produce 2D
//! vector graphics (think SVGs) depicting 3D scenes.
//!
//! *The output of an OpenGL pipeline is a rasterized image. The output of `ln` is
//! a set of 2D vector paths.*
//!
//! ## Quick Start
//!
//! Add `ln` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ln = { git = "https://github.com/fogleman/ln" }
//! ```
//!
//! ## Hello World: A Single Cube
//!
//! Here's a minimal example that renders a cube:
//!
//! ```no_run
//! use ln::{Cube, Scene, Vector};
//!
//! fn main() {
//!     // Create a scene and add a single cube
//!     let mut scene = Scene::new();
//!     scene.add(Cube::new(
//!         Vector::new(-1.0, -1.0, -1.0),
//!         Vector::new(1.0, 1.0, 1.0),
//!     ));
//!
//!     // Define camera parameters
//!     let eye = Vector::new(4.0, 3.0, 2.0);    // camera position
//!     let center = Vector::new(0.0, 0.0, 0.0); // camera looks at
//!     let up = Vector::new(0.0, 0.0, 1.0);     // up direction
//!
//!     // Define rendering parameters
//!     let width = 1024.0;  // rendered width
//!     let height = 1024.0; // rendered height
//!     let fovy = 50.0;     // vertical field of view, degrees
//!     let znear = 0.1;     // near z plane
//!     let zfar = 10.0;     // far z plane
//!     let step = 0.01;     // how finely to chop the paths for visibility testing
//!
//!     // Compute 2D paths that depict the 3D scene
//!     let paths = scene.render(eye, center, up, width, height, fovy, znear, zfar, step);
//!
//!     // Render the paths to an image
//!     paths.write_to_png("out.png", width, height);
//!
//!     // Save the paths as an SVG
//!     paths.write_to_svg("out.svg", width, height).expect("Failed to write SVG");
//! }
//! ```
//!
//! ## Features
//!
//! - **Primitives**: [`Sphere`], [`Cube`], [`Triangle`], [`Cylinder`], [`Cone`], and 3D [`Function`]s
//! - **Triangle Meshes**: Load OBJ files with [`load_obj`] and STL files with [`load_stl`]
//! - **Vector-based "Texturing"**: Customize the surface paths of any shape
//! - **CSG Operations**: Combine shapes with [`new_intersection`] and [`new_difference`]
//! - **Output Formats**: Export to PNG or SVG
//!
//! ## How it Works
//!
//! The rendering process in `ln` is fundamentally different from traditional rasterization:
//!
//! 1. **Path Generation**: Each [`Shape`] provides surface paths via the [`Shape::paths`] method.
//!    These are 3D polylines on the solid's surface.
//!
//! 2. **Visibility Testing**: The paths are subdivided ("chopped") and each point is tested
//!    for visibility by shooting a ray toward the camera using [`Shape::intersect`].
//!
//! 3. **2D Projection**: Visible points are projected to 2D using transformation matrices.
//!
//! 4. **Output**: The resulting 2D paths can be rendered to PNG or SVG.
//!
//! ## The Shape Trait
//!
//! All geometry in `ln` implements the [`Shape`] trait:
//!
//! ```ignore
//! pub trait Shape {
//!     fn compile(&mut self) {}
//!     fn bounding_box(&self) -> Box;
//!     fn contains(&self, v: Vector, f: f64) -> bool;
//!     fn intersect(&self, r: Ray) -> Hit;
//!     fn paths(&self) -> Paths;
//! }
//! ```
//!
//! - [`Shape::bounding_box`]: Returns the axis-aligned bounding box
//! - [`Shape::contains`]: Tests if a point is inside the solid (used for CSG)
//! - [`Shape::intersect`]: Tests ray-solid intersection for visibility
//! - [`Shape::paths`]: Returns the 3D paths to be rendered
//!
//! ## Custom Texturing
//!
//! You can implement the [`Shape`] trait to create custom surface patterns:
//!
//! ```ignore
//! use ln::{Cube, Shape, Paths, Vector, Box, Hit, Ray};
//!
//! struct StripedCube {
//!     cube: Cube,
//!     stripes: i32,
//! }
//!
//! impl Shape for StripedCube {
//!     fn bounding_box(&self) -> Box {
//!         self.cube.bounding_box()
//!     }
//!
//!     fn contains(&self, v: Vector, f: f64) -> bool {
//!         self.cube.contains(v, f)
//!     }
//!
//!     fn intersect(&self, r: Ray) -> Hit {
//!         self.cube.intersect(r)
//!     }
//!
//!     fn paths(&self) -> Paths {
//!         let mut paths = Vec::new();
//!         let (x1, y1, z1) = (self.cube.min.x, self.cube.min.y, self.cube.min.z);
//!         let (x2, y2, z2) = (self.cube.max.x, self.cube.max.y, self.cube.max.z);
//!         
//!         for i in 0..=self.stripes {
//!             let p = i as f64 / self.stripes as f64;
//!             let x = x1 + (x2 - x1) * p;
//!             let y = y1 + (y2 - y1) * p;
//!             paths.push(vec![Vector::new(x, y1, z1), Vector::new(x, y1, z2)]);
//!             paths.push(vec![Vector::new(x, y2, z1), Vector::new(x, y2, z2)]);
//!             paths.push(vec![Vector::new(x1, y, z1), Vector::new(x1, y, z2)]);
//!             paths.push(vec![Vector::new(x2, y, z1), Vector::new(x2, y, z2)]);
//!         }
//!         Paths::from_vec(paths)
//!     }
//! }
//! ```
//!
//! ## Constructive Solid Geometry (CSG)
//!
//! Create complex solids by combining simpler shapes:
//!
//! ```no_run
//! use ln::{
//!     new_difference, new_intersection, radians, Cube, Cylinder,
//!     Matrix, Sphere, TransformedShape, Vector, Scene, Shape,
//! };
//! use std::sync::Arc;
//!
//! // Create a sphere intersected with a cube, minus three cylinders
//! let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 1.0));
//! let cube: Arc<dyn Shape + Send + Sync> = Arc::new(Cube::new(
//!     Vector::new(-0.8, -0.8, -0.8),
//!     Vector::new(0.8, 0.8, 0.8),
//! ));
//!
//! let cyl1: Arc<dyn Shape + Send + Sync> = Arc::new(Cylinder::new(0.4, -2.0, 2.0));
//! let cyl2: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
//!     Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
//!     Matrix::rotate(Vector::new(1.0, 0.0, 0.0), radians(90.0)),
//! ));
//! let cyl3: Arc<dyn Shape + Send + Sync> = Arc::new(TransformedShape::new(
//!     Arc::new(Cylinder::new(0.4, -2.0, 2.0)),
//!     Matrix::rotate(Vector::new(0.0, 1.0, 0.0), radians(90.0)),
//! ));
//!
//! // (Sphere & Cube) - (Cylinder | Cylinder | Cylinder)
//! let shape = new_difference(vec![
//!     new_intersection(vec![sphere, cube]),
//!     cyl1, cyl2, cyl3,
//! ]);
//!
//! let mut scene = Scene::new();
//! scene.add_arc(shape);
//!
//! // Render the scene
//! let eye = Vector::new(0.0, 6.0, 2.0);
//! let center = Vector::new(0.0, 0.0, 0.0);
//! let up = Vector::new(0.0, 0.0, 1.0);
//! let paths = scene.render(eye, center, up, 750.0, 750.0, 20.0, 0.1, 100.0, 0.01);
//! paths.write_to_png("csg.png", 750.0, 750.0);
//! ```
//!
//! ## Loading 3D Models
//!
//! Load OBJ or STL files:
//!
//! ```no_run
//! use ln::{load_obj, Scene, Vector};
//!
//! let mesh = load_obj("model.obj").expect("Failed to load OBJ");
//! let mut scene = Scene::new();
//! scene.add(mesh);
//!
//! let eye = Vector::new(4.0, 3.0, 2.0);
//! let center = Vector::new(0.0, 0.0, 0.0);
//! let up = Vector::new(0.0, 0.0, 1.0);
//! let paths = scene.render(eye, center, up, 1024.0, 1024.0, 50.0, 0.1, 100.0, 0.01);
//! paths.write_to_png("model.png", 1024.0, 1024.0);
//! ```
//!
//! ## Transformations
//!
//! Apply transformations to shapes using [`TransformedShape`] and [`Matrix`]:
//!
//! ```no_run
//! use ln::{radians, Cube, Matrix, Scene, TransformedShape, Vector};
//! use std::sync::Arc;
//!
//! let cube = Arc::new(Cube::new(
//!     Vector::new(-1.0, -1.0, -1.0),
//!     Vector::new(1.0, 1.0, 1.0),
//! ));
//!
//! // Rotate 45 degrees around Z axis and translate
//! let transform = Matrix::rotate(Vector::new(0.0, 0.0, 1.0), radians(45.0))
//!     .translated(Vector::new(3.0, 0.0, 0.0));
//!
//! let mut scene = Scene::new();
//! scene.add(TransformedShape::new(cube, transform));
//! ```
//!
//! ## Modules
//!
//! The library is organized into the following modules:
//!
//! - [`scene`]: Scene management and rendering
//! - [`shape`]: The core [`Shape`] trait and transformations
//! - [`vector`]: 3D vector mathematics
//! - [`matrix`]: 4x4 transformation matrices
//! - [`path`]: 2D/3D path handling and output
//! - [`cube`], [`sphere`], [`cylinder`], [`cone`]: Primitive shapes
//! - [`triangle`], [`mesh`]: Triangle mesh support
//! - [`csg`]: Constructive solid geometry operations
//! - [`obj`], [`stl`]: File format loaders
//! - [`function`]: 3D function surfaces

pub mod axis;
pub mod bounding_box;
pub mod common;
pub mod cone;
pub mod csg;
pub mod cube;
pub mod cylinder;
pub mod filter;
pub mod function;
pub mod hit;
pub mod matrix;
pub mod mesh;
pub mod obj;
pub mod path;
pub mod plane;
pub mod ray;
pub mod scene;
pub mod shape;
pub mod sphere;
pub mod stl;
pub mod tree;
pub mod triangle;
pub mod util;
pub mod vector;

// Re-exports for convenient access
pub use axis::Axis;
pub use bounding_box::Box;
pub use cone::{Cone, OutlineCone};
pub use csg::{new_difference, new_intersection, BooleanShape, Op};
pub use cube::Cube;
pub use cylinder::{new_transformed_outline_cylinder, Cylinder, OutlineCylinder};
pub use filter::{ClipFilter, Filter};
pub use function::{Direction, Function, FunctionTexture};
pub use hit::Hit;
pub use matrix::Matrix;
pub use mesh::Mesh;
pub use obj::load_obj;
pub use path::{Path, Paths};
pub use plane::Plane;
pub use ray::Ray;
pub use scene::Scene;
pub use shape::{EmptyShape, Shape, TransformedShape};
pub use sphere::{lat_lng_to_xyz, OutlineSphere, Sphere, SphereTexture};
pub use stl::{load_binary_stl, load_stl, save_binary_stl};
pub use tree::Tree;
pub use triangle::Triangle;
pub use util::{degrees, median, radians};
pub use vector::Vector;

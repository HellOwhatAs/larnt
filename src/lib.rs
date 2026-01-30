//! ln - The 3D Line Art Engine
//!
//! ln is a vector-based 3D renderer written in Rust. It is used to produce 2D
//! vector graphics (think SVGs) depicting 3D scenes.
//!
//! The output of an OpenGL pipeline is a rastered image. The output of ln is
//! a set of 2D vector paths.

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
pub use cylinder::{Cylinder, OutlineCylinder};
pub use filter::{ClipFilter, Filter};
pub use function::{Direction, Function};
pub use hit::Hit;
pub use matrix::Matrix;
pub use mesh::Mesh;
pub use obj::load_obj;
pub use path::{Path, Paths};
pub use plane::Plane;
pub use ray::Ray;
pub use scene::Scene;
pub use shape::{EmptyShape, Shape, TransformedShape};
pub use sphere::{lat_lng_to_xyz, OutlineSphere, Sphere};
pub use stl::{load_binary_stl, load_stl, save_binary_stl};
pub use tree::Tree;
pub use triangle::Triangle;
pub use util::{degrees, median, radians};
pub use vector::Vector;

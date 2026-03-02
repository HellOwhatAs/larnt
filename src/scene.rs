//! Scene management and rendering.
//!
//! This module provides the [`render`] function, which is the main entry point
//! for rendering a collection of shapes into 2D paths.
//!
//! # Example
//!
//! ```
//! use larnt::{Cube, Vector, render};
//!
//! let cube = Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build();
//!
//! let eye = Vector::new(4.0, 3.0, 2.0);
//! let paths = render(vec![cube]).eye(eye).call();
//! paths.write_to_png("output.png", 1024.0, 1024.0).expect("Failed to write PNG");
//! ```

use crate::filter::ClipFilter;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::tree::Tree;
use crate::vector::Vector;
use bon::builder;

/// Renders a collection of shapes to 2D paths.
///
/// This is the main rendering function. It:
/// 1. Gets all paths from shapes
/// 2. Chops paths adaptively for visibility testing (if `step > 0.0`)
/// 3. Builds a BVH tree and filters out hidden portions
/// 4. Simplifies paths (if `step > 0.0`)
/// 5. Projects to 2D screen space
///
/// # Arguments
///
/// * `shapes` - The shapes to render (passed as the start argument to the builder)
/// * `eye` - Camera position
/// * `center` - Point the camera looks at (default: origin)
/// * `up` - Up direction vector (default: `+Z`)
/// * `width` - Output width in pixels (default: 1024)
/// * `height` - Output height in pixels (default: 1024)
/// * `fovy` - Vertical field of view in degrees (default: 50)
/// * `near` - Near clipping plane distance (default: 0.1)
/// * `far` - Far clipping plane distance (default: 1000)
/// * `step` - Path subdivision step size for visibility testing (default: 1.0)
///
/// # Example
///
/// ```
/// use larnt::{Cube, Vector, render};
///
/// let cube = Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build();
///
/// let paths = render(vec![cube]).eye(Vector::new(4.0, 3.0, 2.0)).call();
/// ```
#[builder]
pub fn render<T: Shape>(
    #[builder(start_fn)] shapes: Vec<T>,
    eye: Vector,
    #[builder(default = Vector::new(0.0, 0.0, 0.0))] center: Vector,
    #[builder(default = Vector::new(0.0, 0.0, 1.0))] up: Vector,
    #[builder(default = 1024.0)] width: f64,
    #[builder(default = 1024.0)] height: f64,
    #[builder(default = 50.0)] fovy: f64,
    #[builder(default = 0.1)] near: f64,
    #[builder(default = 1e3)] far: f64,
    #[builder(default = 1.0)] step: f64,
) -> Paths<Vector> {
    let aspect = width / height;
    let matrix = Matrix::look_at(eye, center, up);
    let matrix = matrix.with_perspective(fovy, aspect, near, far);

    let viewport_mat = Matrix::translate(Vector::new(1.0, 1.0, 0.0)).scaled(Vector::new(
        width / 2.0,
        height / 2.0,
        1.0,
    ));

    let args = RenderArgs {
        screen_mat: viewport_mat.mul(&matrix),
        eye,
        up,
        width,
        height,
        step,
    };

    let mut paths = Paths::new();
    for shape in shapes.iter() {
        paths.extend(shape.paths(&args));
    }

    if step > 0.0 {
        paths = paths.chop_adaptive(&args);
    }

    let tree = Tree::new(shapes);
    let filter = {
        let visible = |eye: Vector, point: Vector| -> bool {
            let v = eye.sub(point);
            if v.length() == 0.0 {
                return true;
            }
            let r = Ray::new(point, v.normalize());
            let hit = tree.intersect(r);
            hit.t >= v.length()
        };
        ClipFilter::new(matrix, eye, visible)
    };
    paths = paths.filter(&filter);

    if step > 0.0 {
        paths = paths.simplify(1e-6);
    }

    paths.transform(&viewport_mat)
}

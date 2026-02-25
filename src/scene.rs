//! Scene management and rendering.
//!
//! This module provides the [`Scene`] struct, which is the main container for
//! 3D objects and handles the rendering pipeline.
//!
//! # Example
//!
//! ```
//! use larnt::{Cube, Scene, Vector};
//!
//! let mut scene = Scene::new();
//! scene.add(Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build());
//!
//! let eye = Vector::new(4.0, 3.0, 2.0);
//! let paths = scene.render(eye).call();
//! paths.write_to_png("output.png", 1024.0, 1024.0);
//! ```

use crate::filter::ClipFilter;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::{RenderArgs, Shape};
use crate::tree::Tree;
use crate::vector::Vector;
use bon::builder;

/// Renders the scene to 2D paths.
///
/// This is the main rendering function. It:
/// 1. Compiles the BVH tree if needed
/// 2. Gets all paths from shapes
/// 3. Chops paths for visibility testing
/// 4. Filters out hidden portions
/// 5. Projects to 2D screen space
///
/// # Arguments
///
/// * `eye` - Camera position
/// * `center` - Point the camera looks at
/// * `up` - Up direction vector
/// * `width` - Output width in pixels
/// * `height` - Output height in pixels
/// * `fovy` - Vertical field of view in degrees
/// * `near` - Near clipping plane distance
/// * `far` - Far clipping plane distance
/// * `step` - Path subdivision step size for visibility testing
///
/// # Example
///
/// ```
/// use larnt::{Scene, Cube, Vector};
///
/// let mut scene = Scene::new();
/// scene.add(Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build());
///
/// let paths = scene.render(Vector::new(4.0, 3.0, 2.0)).call();
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
) -> Paths {
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

    paths = {
        let tree = Tree::new(shapes);
        let visible = |eye: Vector, point: Vector| -> bool {
            let v = eye.sub(point);
            if v.length() == 0.0 {
                return true;
            }
            let r = Ray::new(point, v.normalize());
            let hit = tree.intersect(r);
            hit.t >= v.length()
        };
        let filter = ClipFilter::new(matrix, eye, visible);
        paths.filter(&filter)
    };

    if step > 0.0 {
        paths = paths.simplify(1e-6);
    }

    paths.transform(&viewport_mat)
}

//! Scene management and rendering.
//!
//! This module provides the [`Scene`] struct, which is the main container for
//! 3D objects and handles the rendering pipeline.
//!
//! # Example
//!
//! ```no_run
//! use ln::{Cube, Scene, Vector};
//!
//! let mut scene = Scene::new();
//! scene.add(Cube::new(
//!     Vector::new(-1.0, -1.0, -1.0),
//!     Vector::new(1.0, 1.0, 1.0),
//! ));
//!
//! let eye = Vector::new(4.0, 3.0, 2.0);
//! let center = Vector::new(0.0, 0.0, 0.0);
//! let up = Vector::new(0.0, 0.0, 1.0);
//!
//! let paths = scene.render(eye, center, up, 1024.0, 1024.0, 50.0, 0.1, 10.0, 0.01);
//! paths.write_to_png("output.png", 1024.0, 1024.0);
//! ```

use crate::filter::ClipFilter;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::tree::Tree;
use crate::vector::Vector;
use std::sync::Arc;

/// A container for 3D shapes that handles rendering.
///
/// The `Scene` struct collects shapes and provides methods to render them
/// into 2D vector paths. It uses a bounding volume hierarchy (BVH) tree
/// for efficient ray-shape intersection tests.
///
/// # Example
///
/// ```no_run
/// use ln::{Scene, Sphere, Vector};
///
/// let mut scene = Scene::new();
/// scene.add(Sphere::new(Vector::new(0.0, 0.0, 0.0), 1.0));
///
/// let paths = scene.render(
///     Vector::new(4.0, 3.0, 2.0),  // eye position
///     Vector::new(0.0, 0.0, 0.0),  // look at
///     Vector::new(0.0, 0.0, 1.0),  // up direction
///     1024.0, 1024.0,              // width, height
///     50.0,                         // field of view (degrees)
///     0.1, 10.0,                    // near and far clip planes
///     0.01,                         // path chopping step
/// );
/// ```
pub struct Scene {
    /// The shapes in this scene.
    pub shapes: Vec<Arc<dyn Shape + Send + Sync>>,
    /// The BVH tree for efficient intersection testing.
    pub tree: Option<Tree>,
}

impl Scene {
    /// Creates a new empty scene.
    pub fn new() -> Self {
        Scene {
            shapes: Vec::new(),
            tree: None,
        }
    }

    /// Compiles the scene by building the BVH tree.
    ///
    /// This is called automatically by [`Scene::render`], but can be called
    /// manually if you want to reuse the same scene for multiple renders.
    pub fn compile(&mut self) {
        if self.tree.is_none() {
            self.tree = Some(Tree::new(self.shapes.clone()));
        }
    }

    /// Adds a shape to the scene.
    ///
    /// The shape is wrapped in an `Arc` and stored for rendering.
    ///
    /// # Example
    ///
    /// ```
    /// use ln::{Scene, Cube, Vector};
    ///
    /// let mut scene = Scene::new();
    /// scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));
    /// ```
    pub fn add<S: Shape + Send + Sync + 'static>(&mut self, shape: S) {
        self.shapes.push(Arc::new(shape));
    }

    /// Adds a pre-wrapped shape to the scene.
    ///
    /// Use this when you already have an `Arc<dyn Shape>`, such as when
    /// working with CSG operations or transformed shapes.
    ///
    /// # Example
    ///
    /// ```
    /// use ln::{Scene, Sphere, Shape, Vector};
    /// use std::sync::Arc;
    ///
    /// let sphere: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new(Vector::default(), 1.0));
    /// let mut scene = Scene::new();
    /// scene.add_arc(sphere);
    /// ```
    pub fn add_arc(&mut self, shape: Arc<dyn Shape + Send + Sync>) {
        self.shapes.push(shape);
    }

    /// Tests for ray-scene intersection.
    ///
    /// Returns a [`Hit`] describing the intersection, or [`Hit::no_hit()`]
    /// if the ray doesn't hit any shape.
    pub fn intersect(&self, r: Ray) -> Hit {
        self.tree.as_ref().map_or(Hit::no_hit(), |tree| tree.intersect(r))
    }

    /// Tests if a point is visible from the camera position.
    ///
    /// Returns `true` if there is no shape blocking the view from
    /// `eye` to `point`.
    pub fn visible(&self, eye: Vector, point: Vector) -> bool {
        let v = eye.sub(point);
        let r = Ray::new(point, v.normalize());
        let hit = self.intersect(r);
        hit.t >= v.length()
    }

    /// Returns all paths from all shapes in the scene.
    pub fn paths(&self) -> Paths {
        let mut result = Paths::new();
        for shape in &self.shapes {
            result.extend(shape.paths());
        }
        result
    }

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
    /// ```no_run
    /// use ln::{Scene, Cube, Vector};
    ///
    /// let mut scene = Scene::new();
    /// scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));
    ///
    /// let paths = scene.render(
    ///     Vector::new(4.0, 3.0, 2.0),
    ///     Vector::new(0.0, 0.0, 0.0),
    ///     Vector::new(0.0, 0.0, 1.0),
    ///     1024.0, 1024.0,
    ///     50.0, 0.1, 10.0, 0.01,
    /// );
    /// ```
    pub fn render(
        &mut self,
        eye: Vector,
        center: Vector,
        up: Vector,
        width: f64,
        height: f64,
        fovy: f64,
        near: f64,
        far: f64,
        step: f64,
    ) -> Paths {
        let aspect = width / height;
        let matrix = Matrix::look_at(eye, center, up);
        let matrix = matrix.with_perspective(fovy, aspect, near, far);
        self.render_with_matrix(matrix, eye, width, height, step)
    }

    /// Renders the scene with a custom transformation matrix.
    ///
    /// This gives you full control over the projection matrix, useful for
    /// orthographic projections or custom camera setups.
    pub fn render_with_matrix(
        &mut self,
        matrix: Matrix,
        eye: Vector,
        width: f64,
        height: f64,
        step: f64,
    ) -> Paths {
        self.compile();
        let mut paths = self.paths();
        
        if step > 0.0 {
            paths = paths.chop(step);
        }
        
        let filter = ClipFilter {
            matrix,
            eye,
            scene: self,
        };
        paths = paths.filter(&filter);
        
        if step > 0.0 {
            paths = paths.simplify(1e-6);
        }
        
        let matrix = Matrix::translate(Vector::new(1.0, 1.0, 0.0))
            .scaled(Vector::new(width / 2.0, height / 2.0, 0.0));
        paths = paths.transform(&matrix);
        
        paths
    }
}

impl Default for Scene {
    fn default() -> Self {
        Scene::new()
    }
}

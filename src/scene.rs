use crate::filter::ClipFilter;
use crate::hit::Hit;
use crate::matrix::Matrix;
use crate::path::Paths;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::tree::Tree;
use crate::vector::Vector;
use std::sync::Arc;

pub struct Scene {
    pub shapes: Vec<Arc<dyn Shape + Send + Sync>>,
    pub tree: Option<Tree>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            shapes: Vec::new(),
            tree: None,
        }
    }

    pub fn compile(&mut self) {
        if self.tree.is_none() {
            self.tree = Some(Tree::new(self.shapes.clone()));
        }
    }

    pub fn add<S: Shape + Send + Sync + 'static>(&mut self, shape: S) {
        self.shapes.push(Arc::new(shape));
    }

    pub fn add_arc(&mut self, shape: Arc<dyn Shape + Send + Sync>) {
        self.shapes.push(shape);
    }

    pub fn intersect(&self, r: Ray) -> Hit {
        self.tree.as_ref().map_or(Hit::no_hit(), |tree| tree.intersect(r))
    }

    pub fn visible(&self, eye: Vector, point: Vector) -> bool {
        let v = eye.sub(point);
        let r = Ray::new(point, v.normalize());
        let hit = self.intersect(r);
        hit.t >= v.length()
    }

    pub fn paths(&self) -> Paths {
        let mut result = Paths::new();
        for shape in &self.shapes {
            result.extend(shape.paths());
        }
        result
    }

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

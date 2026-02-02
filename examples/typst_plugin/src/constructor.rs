use crate::eval_func;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub enum Matrix {
    Rotate { v: [f64; 3], a: f64 },
    Scale { v: [f64; 3] },
    Translate { v: [f64; 3] },
}

impl Matrix {
    fn to_matrix(self) -> ln::Matrix {
        match self {
            Matrix::Rotate { v, a } => ln::Matrix::rotate(ln::Vector::new(v[0], v[1], v[2]), a),
            Matrix::Scale { v } => ln::Matrix::scale(ln::Vector::new(v[0], v[1], v[2])),
            Matrix::Translate { v } => ln::Matrix::translate(ln::Vector::new(v[0], v[1], v[2])),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum LnShape {
    Cone {
        radius: f64,
        v0: [f64; 3],
        v1: [f64; 3],
    },
    Cube {
        min: [f64; 3],
        max: [f64; 3],
        texture: String,
        stripes: Option<u64>,
    },
    Cylinder {
        radius: f64,
        v0: [f64; 3],
        v1: [f64; 3],
    },
    Sphere {
        center: [f64; 3],
        radius: f64,
        texture: String,
        seed: Option<u64>,
    },
    Function {
        func: String,
        bbox: ([f64; 3], [f64; 3]),
        direction: String,
        texture: String,
    },
    Triangle {
        v1: [f64; 3],
        v2: [f64; 3],
        v3: [f64; 3],
    },
    Mesh(Vec<LnShape>),

    Outline(Box<LnShape>), // Cone, Cylinder, Sphere
    Difference(Vec<LnShape>),
    Intersection(Vec<LnShape>),
    Transformation {
        shape: Box<LnShape>,
        matrix: Matrix,
    },
}

impl LnShape {
    pub fn to_shape(
        self,
        eye: ln::Vector,
        up: ln::Vector,
    ) -> Result<Arc<dyn ln::Shape + Send + Sync>, String> {
        Ok(match self {
            LnShape::Cone { radius, v0, v1 } => Arc::new(ln::new_transformed_cone(
                up,
                ln::Vector::new(v0[0], v0[1], v0[2]),
                ln::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )),
            LnShape::Cube {
                min,
                max,
                texture,
                stripes,
            } => {
                let min_v = ln::Vector::new(min[0], min[1], min[2]);
                let max_v = ln::Vector::new(max[0], max[1], max[2]);
                Arc::new(ln::Cube::new(min_v, max_v).with_texture(
                    match (texture.as_str(), stripes) {
                        ("Vanilla", _) => ln::CubeTexture::Vanilla,
                        ("Stripes", Some(n)) => ln::CubeTexture::Striped(n),
                        _ => {
                            return Err(format!(
                                "Invalid cube texture: {}, stripes: {:?}",
                                texture, stripes
                            ));
                        }
                    },
                ))
            }
            LnShape::Cylinder { radius, v0, v1 } => Arc::new(ln::new_transformed_cylinder(
                up,
                ln::Vector::new(v0[0], v0[1], v0[2]),
                ln::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )),
            LnShape::Sphere {
                center,
                radius,
                texture,
                seed,
            } => {
                let center_v = ln::Vector::new(center[0], center[1], center[2]);
                let sphere = ln::Sphere::new(center_v, radius);
                let sphere = match (texture.as_str(), seed) {
                    ("LatLng", _) => sphere.with_texture(ln::SphereTexture::LatLng),
                    ("RandomEquators", Some(seed)) => {
                        sphere.with_texture(ln::SphereTexture::RandomEquators(seed))
                    }
                    ("RandomDots", Some(seed)) => {
                        sphere.with_texture(ln::SphereTexture::RandomDots(seed))
                    }
                    ("RandomCircles", Some(seed)) => {
                        sphere.with_texture(ln::SphereTexture::RandomCircles(seed))
                    }
                    _ => {
                        return Err(format!(
                            "Invalid sphere parameters: center: {:?}, radius: {}, texture: {}, seed: {:?}",
                            center, radius, texture, seed
                        ));
                    }
                };
                Arc::new(sphere)
            }
            LnShape::Function {
                func,
                bbox,
                direction,
                texture,
            } => {
                let (slab, compiled) = eval_func::str2compiled(&func).map_err(|e| e.to_string())?;
                let f = eval_func::compiled2func(slab, compiled);
                let func = ln::Function::new(
                    f,
                    ln::Box::new(
                        ln::Vector::new(bbox.0[0], bbox.0[1], bbox.0[2]),
                        ln::Vector::new(bbox.1[0], bbox.1[1], bbox.1[2]),
                    ),
                    match direction.as_str() {
                        "Below" => ln::Direction::Below,
                        "Above" => ln::Direction::Above,
                        _ => {
                            return Err(format!(
                                "Invalid function direction: {}. Must be 'Below' or 'Above'",
                                direction
                            ));
                        }
                    },
                )
                .with_texture(match texture.as_str() {
                    "Grid" => ln::FunctionTexture::Grid,
                    "Swirl" => ln::FunctionTexture::Swirl,
                    "Spiral" => ln::FunctionTexture::Spiral,
                    _ => {
                        return Err(format!(
                            "Invalid function texture: {}. Must be 'Grid', 'Swirl', or 'Spiral'",
                            texture
                        ));
                    }
                });
                Arc::new(func)
            }
            LnShape::Triangle { v1, v2, v3 } => {
                let v1_v = ln::Vector::new(v1[0], v1[1], v1[2]);
                let v2_v = ln::Vector::new(v2[0], v2[1], v2[2]);
                let v3_v = ln::Vector::new(v3[0], v3[1], v3[2]);
                Arc::new(ln::Triangle::new(v1_v, v2_v, v3_v))
            }
            LnShape::Mesh(ln_shapes) => {
                let mut triangles = Vec::new();
                for lnshape in ln_shapes {
                    if let LnShape::Triangle { v1, v2, v3 } = &lnshape {
                        triangles.push(ln::Triangle::new(
                            ln::Vector::new(v1[0], v1[1], v1[2]),
                            ln::Vector::new(v2[0], v2[1], v2[2]),
                            ln::Vector::new(v3[0], v3[1], v3[2]),
                        ));
                    } else {
                        return Err("Mesh can only contain Triangle shapes".to_string());
                    }
                }
                Arc::new(ln::Mesh::new(triangles))
            }
            LnShape::Outline(ln_shape) => match *ln_shape {
                LnShape::Cone { radius, v0, v1 } => Arc::new(ln::new_transformed_outline_cone(
                    eye,
                    up,
                    ln::Vector::new(v0[0], v0[1], v0[2]),
                    ln::Vector::new(v1[0], v1[1], v1[2]),
                    radius,
                )),
                LnShape::Cylinder { radius, v0, v1 } => {
                    Arc::new(ln::new_transformed_outline_cylinder(
                        eye,
                        up,
                        ln::Vector::new(v0[0], v0[1], v0[2]),
                        ln::Vector::new(v1[0], v1[1], v1[2]),
                        radius,
                    ))
                }
                LnShape::Sphere {
                    center,
                    radius,
                    texture: _,
                    seed: _,
                } => Arc::new(ln::OutlineSphere::new(
                    eye,
                    up,
                    ln::Vector::new(center[0], center[1], center[2]),
                    radius,
                )),
                _ => {
                    return Err(
                        "Outline can only be applied to Cone, Cylinder, or Sphere shapes"
                            .to_string(),
                    );
                }
            },
            LnShape::Difference(ln_shapes) => {
                let shapes = ln_shapes
                    .into_iter()
                    .map(|s| s.to_shape(eye, up))
                    .collect::<Result<Vec<_>, _>>()?;
                ln::new_difference(shapes)
            }
            LnShape::Intersection(ln_shapes) => {
                let shapes = ln_shapes
                    .into_iter()
                    .map(|s| s.to_shape(eye, up))
                    .collect::<Result<Vec<_>, _>>()?;
                ln::new_intersection(shapes)
            }
            LnShape::Transformation { shape, matrix } => Arc::new(ln::TransformedShape::new(
                shape.to_shape(matrix.clone().to_matrix().inverse().mul_position(eye), up)?,
                matrix.to_matrix(),
            )),
        })
    }
}

pub fn render(
    shapes: impl Iterator<Item = LnShape>,
    eye: [f64; 3],
    center: [f64; 3],
    up: [f64; 3],
    width: f64,
    height: f64,
    fovy: f64,
    near: f64,
    far: f64,
    step: f64,
) -> Result<ln::Paths, String> {
    let eye = ln::Vector::new(eye[0], eye[1], eye[2]);
    let center = ln::Vector::new(center[0], center[1], center[2]);
    let up = ln::Vector::new(up[0], up[1], up[2]);

    let mut scene = ln::Scene::new();
    for shape in shapes {
        scene.add_arc(shape.to_shape(eye, up)?);
    }
    Ok(scene.render(eye, center, up, width, height, fovy, near, far, step))
}

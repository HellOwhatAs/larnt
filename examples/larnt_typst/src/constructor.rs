use crate::interp;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub enum Matrix {
    Rotate { v: [f64; 3], a: f64 },
    Scale { v: [f64; 3] },
    Translate { v: [f64; 3] },
}

impl Matrix {
    fn to_matrix(self) -> larnt::Matrix {
        match self {
            Matrix::Rotate { v, a } => larnt::Matrix::rotate(larnt::Vector::new(v[0], v[1], v[2]), a),
            Matrix::Scale { v } => larnt::Matrix::scale(larnt::Vector::new(v[0], v[1], v[2])),
            Matrix::Translate { v } => larnt::Matrix::translate(larnt::Vector::new(v[0], v[1], v[2])),
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
        samples: Vec<Vec<f64>>,
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
        eye: larnt::Vector,
        up: larnt::Vector,
    ) -> Result<Arc<dyn larnt::Shape + Send + Sync>, String> {
        Ok(match self {
            LnShape::Cone { radius, v0, v1 } => Arc::new(larnt::new_transformed_cone(
                up,
                larnt::Vector::new(v0[0], v0[1], v0[2]),
                larnt::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )),
            LnShape::Cube {
                min,
                max,
                texture,
                stripes,
            } => {
                let min_v = larnt::Vector::new(min[0], min[1], min[2]);
                let max_v = larnt::Vector::new(max[0], max[1], max[2]);
                Arc::new(larnt::Cube::new(min_v, max_v).with_texture(
                    match (texture.as_str(), stripes) {
                        ("Vanilla", _) => larnt::CubeTexture::Vanilla,
                        ("Stripes", Some(n)) => larnt::CubeTexture::Striped(n),
                        _ => {
                            return Err(format!(
                                "Invalid cube texture: {}, stripes: {:?}",
                                texture, stripes
                            ));
                        }
                    },
                ))
            }
            LnShape::Cylinder { radius, v0, v1 } => Arc::new(larnt::new_transformed_cylinder(
                up,
                larnt::Vector::new(v0[0], v0[1], v0[2]),
                larnt::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )),
            LnShape::Sphere {
                center,
                radius,
                texture,
                seed,
            } => {
                let center_v = larnt::Vector::new(center[0], center[1], center[2]);
                let sphere = larnt::Sphere::new(center_v, radius);
                let sphere = match (texture.as_str(), seed) {
                    ("LatLng", _) => sphere.with_texture(larnt::SphereTexture::LatLng),
                    ("RandomEquators", Some(seed)) => {
                        sphere.with_texture(larnt::SphereTexture::RandomEquators(seed))
                    }
                    ("RandomDots", Some(seed)) => {
                        sphere.with_texture(larnt::SphereTexture::RandomDots(seed))
                    }
                    ("RandomCircles", Some(seed)) => {
                        sphere.with_texture(larnt::SphereTexture::RandomCircles(seed))
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
                samples,
                bbox,
                direction,
                texture,
            } => {
                if samples.len() < 2 || samples[0].len() < 2 {
                    return Err("Function samples must be at least 2x2".to_string());
                }
                if samples.iter().any(|row| row.len() != samples[0].len()) {
                    return Err("Function samples must have consistent row lengths".to_string());
                }
                let grid = interp::BilinearGrid::new(
                    samples[0].len(),
                    samples.len(),
                    samples.into_iter().flatten().collect(),
                    (bbox.0[0], bbox.1[0]),
                    (bbox.0[1], bbox.1[1]),
                );
                let func = larnt::Function::new(
                    move |x, y| grid.get(x, y),
                    larnt::Box::new(
                        larnt::Vector::new(bbox.0[0], bbox.0[1], bbox.0[2]),
                        larnt::Vector::new(bbox.1[0], bbox.1[1], bbox.1[2]),
                    ),
                    match direction.as_str() {
                        "Below" => larnt::Direction::Below,
                        "Above" => larnt::Direction::Above,
                        _ => {
                            return Err(format!(
                                "Invalid function direction: {}. Must be 'Below' or 'Above'",
                                direction
                            ));
                        }
                    },
                )
                .with_texture(match texture.as_str() {
                    "Grid" => larnt::FunctionTexture::Grid,
                    "Swirl" => larnt::FunctionTexture::Swirl,
                    "Spiral" => larnt::FunctionTexture::Spiral,
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
                let v1_v = larnt::Vector::new(v1[0], v1[1], v1[2]);
                let v2_v = larnt::Vector::new(v2[0], v2[1], v2[2]);
                let v3_v = larnt::Vector::new(v3[0], v3[1], v3[2]);
                Arc::new(larnt::Triangle::new(v1_v, v2_v, v3_v))
            }
            LnShape::Mesh(ln_shapes) => {
                let mut triangles = Vec::new();
                for lnshape in ln_shapes {
                    if let LnShape::Triangle { v1, v2, v3 } = &lnshape {
                        triangles.push(larnt::Triangle::new(
                            larnt::Vector::new(v1[0], v1[1], v1[2]),
                            larnt::Vector::new(v2[0], v2[1], v2[2]),
                            larnt::Vector::new(v3[0], v3[1], v3[2]),
                        ));
                    } else {
                        return Err("Mesh can only contain Triangle shapes".to_string());
                    }
                }
                Arc::new(larnt::Mesh::new(triangles))
            }
            LnShape::Outline(ln_shape) => match *ln_shape {
                LnShape::Cone { radius, v0, v1 } => Arc::new(larnt::new_transformed_outline_cone(
                    eye,
                    up,
                    larnt::Vector::new(v0[0], v0[1], v0[2]),
                    larnt::Vector::new(v1[0], v1[1], v1[2]),
                    radius,
                )),
                LnShape::Cylinder { radius, v0, v1 } => {
                    Arc::new(larnt::new_transformed_outline_cylinder(
                        eye,
                        up,
                        larnt::Vector::new(v0[0], v0[1], v0[2]),
                        larnt::Vector::new(v1[0], v1[1], v1[2]),
                        radius,
                    ))
                }
                LnShape::Sphere {
                    center,
                    radius,
                    texture: _,
                    seed: _,
                } => Arc::new(larnt::OutlineSphere::new(
                    eye,
                    up,
                    larnt::Vector::new(center[0], center[1], center[2]),
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
                larnt::new_difference(shapes)
            }
            LnShape::Intersection(ln_shapes) => {
                let shapes = ln_shapes
                    .into_iter()
                    .map(|s| s.to_shape(eye, up))
                    .collect::<Result<Vec<_>, _>>()?;
                larnt::new_intersection(shapes)
            }
            LnShape::Transformation { shape, matrix } => Arc::new(larnt::TransformedShape::new(
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
) -> Result<larnt::Paths, String> {
    let eye = larnt::Vector::new(eye[0], eye[1], eye[2]);
    let center = larnt::Vector::new(center[0], center[1], center[2]);
    let up = larnt::Vector::new(up[0], up[1], up[2]);

    let mut scene = larnt::Scene::new();
    for shape in shapes {
        scene.add_arc(shape.to_shape(eye, up)?);
    }
    Ok(scene.render(eye, center, up, width, height, fovy, near, far, step))
}

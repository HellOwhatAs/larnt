use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Matrix {
    Rotate { v: [f64; 3], a: f64 },
    Scale { v: [f64; 3] },
    Translate { v: [f64; 3] },
    Raw([f64; 16]),
}

impl Matrix {
    fn to_matrix(self) -> larnt::Matrix {
        match self {
            Matrix::Rotate { v, a } => {
                larnt::Matrix::rotate(larnt::Vector::new(v[0], v[1], v[2]), a)
            }
            Matrix::Scale { v } => larnt::Matrix::scale(larnt::Vector::new(v[0], v[1], v[2])),
            Matrix::Translate { v } => {
                larnt::Matrix::translate(larnt::Vector::new(v[0], v[1], v[2]))
            }
            Matrix::Raw(m) => larnt::Matrix {
                x00: m[0],
                x01: m[1],
                x02: m[2],
                x03: m[3],
                x10: m[4],
                x11: m[5],
                x12: m[6],
                x13: m[7],
                x20: m[8],
                x21: m[9],
                x22: m[10],
                x23: m[11],
                x30: m[12],
                x31: m[13],
                x32: m[14],
                x33: m[15],
            },
        }
    }

    fn from_raw(mat: larnt::Matrix) -> Self {
        Matrix::Raw([
            mat.x00, mat.x01, mat.x02, mat.x03, mat.x10, mat.x11, mat.x12, mat.x13, mat.x20,
            mat.x21, mat.x22, mat.x23, mat.x30, mat.x31, mat.x32, mat.x33,
        ])
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum ConeTexture {
    #[default]
    Outline,
    Striped(u64),
}

impl ConeTexture {
    fn to_texture(self) -> larnt::ConeTexture {
        match self {
            ConeTexture::Outline => larnt::ConeTexture::Outline,
            ConeTexture::Striped(n) => larnt::ConeTexture::Striped(n),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum CubeTexture {
    #[default]
    Vanilla,
    Striped(u64),
}

impl CubeTexture {
    fn to_texture(self) -> larnt::CubeTexture {
        match self {
            CubeTexture::Vanilla => larnt::CubeTexture::Vanilla,
            CubeTexture::Striped(n) => larnt::CubeTexture::Striped(n),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum CylinderTexture {
    #[default]
    Outline,
    Striped(u64),
}

impl CylinderTexture {
    fn to_texture(self) -> larnt::CylinderTexture {
        match self {
            CylinderTexture::Outline => larnt::CylinderTexture::Outline,
            CylinderTexture::Striped(n) => larnt::CylinderTexture::Striped(n),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum SphereTexture {
    #[default]
    Outline,
    LatLng {
        n: i32,
        o: i32,
    },
    RandomEquators {
        seed: u64,
        n: usize,
    },
    RandomFuzz {
        seed: u64,
        num: usize,
        scale: f64,
    },
    RandomCircles {
        seed: u64,
        num: usize,
    },
}

impl SphereTexture {
    fn to_texture(self) -> larnt::SphereTexture {
        match self {
            SphereTexture::Outline => larnt::SphereTexture::Outline,
            SphereTexture::LatLng { n, o } => larnt::SphereTexture::LatLng { n, o },
            SphereTexture::RandomEquators { seed, n } => {
                larnt::SphereTexture::RandomEquators { seed, n }
            }
            SphereTexture::RandomFuzz { seed, num, scale } => {
                larnt::SphereTexture::RandomFuzz { seed, num, scale }
            }
            SphereTexture::RandomCircles { seed, num } => {
                larnt::SphereTexture::RandomCircles { seed, num }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum MeshTexture {
    #[default]
    Triangles,
    Polygonal,
    Silhouette,
}

impl MeshTexture {
    fn to_texture(self) -> larnt::MeshTexture {
        match self {
            MeshTexture::Triangles => larnt::MeshTexture::Triangles,
            MeshTexture::Polygonal => larnt::MeshTexture::Polygonal,
            MeshTexture::Silhouette => larnt::MeshTexture::Silhouette,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum ParametricSurfaceTexture {
    #[default]
    Grid,
    Triangles,
    Polygonal,
    Silhouette,
}

impl ParametricSurfaceTexture {
    fn to_texture(self) -> Option<larnt::MeshTexture> {
        match self {
            ParametricSurfaceTexture::Grid => None,
            ParametricSurfaceTexture::Triangles => Some(larnt::MeshTexture::Triangles),
            ParametricSurfaceTexture::Polygonal => Some(larnt::MeshTexture::Polygonal),
            ParametricSurfaceTexture::Silhouette => Some(larnt::MeshTexture::Silhouette),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum LnShape {
    Cone {
        radius: f64,
        v0: [f64; 3],
        v1: [f64; 3],
        texture: ConeTexture,
    },
    Cube {
        min: [f64; 3],
        max: [f64; 3],
        texture: CubeTexture,
    },
    Cylinder {
        radius: f64,
        v0: [f64; 3],
        v1: [f64; 3],
        texture: CylinderTexture,
    },
    Sphere {
        center: [f64; 3],
        radius: f64,
        texture: SphereTexture,
    },
    Triangle {
        v1: [f64; 3],
        v2: [f64; 3],
        v3: [f64; 3],
    },
    Mesh {
        vertices: Vec<[f64; 3]>,
        triangles: Vec<usize>,
        flipped_triangles: Vec<(usize, usize)>,
        texture: MeshTexture,
    },
    ParametricSurface {
        samples: Vec<(f64, f64, f64)>,
        u_steps: usize,
        v_steps: usize,
        texture: ParametricSurfaceTexture,
    },

    Difference(Vec<LnShape>),
    Intersection(Vec<LnShape>),
    Transformation {
        shape: Box<LnShape>,
        matrix: Matrix,
    },
}

impl LnShape {
    pub fn to_shape(self) -> Result<larnt::Primitive, String> {
        Ok(match self {
            LnShape::Cone {
                radius,
                v0,
                v1,
                texture,
            } => larnt::new_transformed_cone(
                larnt::Vector::new(v0[0], v0[1], v0[2]),
                larnt::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )
            .texture(texture.to_texture())
            .call()
            .into(),
            LnShape::Cube { min, max, texture } => {
                let min_v = larnt::Vector::new(min[0], min[1], min[2]);
                let max_v = larnt::Vector::new(max[0], max[1], max[2]);
                larnt::Cube::builder(min_v, max_v)
                    .texture(texture.to_texture())
                    .build()
                    .into()
            }
            LnShape::Cylinder {
                radius,
                v0,
                v1,
                texture,
            } => larnt::new_transformed_cylinder(
                larnt::Vector::new(v0[0], v0[1], v0[2]),
                larnt::Vector::new(v1[0], v1[1], v1[2]),
                radius,
            )
            .texture(texture.to_texture())
            .call()
            .into(),
            LnShape::Sphere {
                center,
                radius,
                texture,
            } => {
                let center_v = larnt::Vector::new(center[0], center[1], center[2]);
                larnt::Sphere::builder(center_v, radius)
                    .texture(texture.to_texture())
                    .build()
                    .into()
            }
            LnShape::Triangle { v1, v2, v3 } => {
                let v1_v = larnt::Vector::new(v1[0], v1[1], v1[2]);
                let v2_v = larnt::Vector::new(v2[0], v2[1], v2[2]);
                let v3_v = larnt::Vector::new(v3[0], v3[1], v3[2]);
                larnt::Triangle::new(v1_v, v2_v, v3_v).into()
            }
            LnShape::Mesh {
                vertices,
                triangles,
                flipped_triangles,
                texture,
            } => larnt::Mesh::builder(
                vertices
                    .into_iter()
                    .map(|[x, y, z]| larnt::Vector::new(x, y, z))
                    .collect(),
                triangles,
            )
            .flipped_triangles(flipped_triangles.into_iter().collect())
            .texture(texture.to_texture())
            .build()
            .into(),
            LnShape::ParametricSurface {
                samples,
                u_steps,
                v_steps,
                texture,
            } => {
                if let Some(texture) = texture.to_texture() {
                    let mut mesh = larnt::ParametricSurface::mesh_from_grid(
                        samples
                            .into_iter()
                            .map(|(x, y, z)| larnt::Vector::new(x, y, z))
                            .collect(),
                        u_steps,
                        v_steps,
                        |i, j| i * (v_steps + 1) + j,
                    );
                    mesh.texture = texture;
                    mesh.into()
                } else {
                    larnt::ParametricSurface::from_grid(
                        samples
                            .into_iter()
                            .map(|(x, y, z)| larnt::Vector::new(x, y, z))
                            .collect(),
                        u_steps,
                        v_steps,
                        |i, j| i * (v_steps + 1) + j,
                    )
                    .into()
                }
            }
            LnShape::Difference(ln_shapes) => {
                let shapes = ln_shapes
                    .into_iter()
                    .map(|s| s.to_shape())
                    .collect::<Result<Vec<_>, _>>()?;
                larnt::new_difference(shapes)
            }
            LnShape::Intersection(ln_shapes) => {
                let shapes = ln_shapes
                    .into_iter()
                    .map(|s| s.to_shape())
                    .collect::<Result<Vec<_>, _>>()?;
                larnt::new_intersection(shapes)
            }
            LnShape::Transformation { shape, matrix } => {
                if let LnShape::Transformation {
                    shape: shape_inner,
                    matrix: matrix_inner,
                } = *shape
                {
                    let res = LnShape::Transformation {
                        shape: shape_inner,
                        matrix: Matrix::from_raw(matrix.to_matrix().mul(&matrix_inner.to_matrix())),
                    }
                    .to_shape();
                    match res {
                        Ok(s) => s,
                        Err(e) => return Err(e),
                    }
                } else {
                    larnt::TransformedShape::new(shape.to_shape()?, matrix.to_matrix()).into()
                }
            }
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
) -> Result<larnt::Paths<larnt::Vector>, String> {
    let eye = larnt::Vector::new(eye[0], eye[1], eye[2]);
    let center = larnt::Vector::new(center[0], center[1], center[2]);
    let up = larnt::Vector::new(up[0], up[1], up[2]);

    Ok(larnt::render(
        shapes
            .into_iter()
            .map(|shape| shape.to_shape())
            .collect::<Result<Vec<larnt::Primitive>, String>>()?,
    )
    .eye(eye)
    .center(center)
    .up(up)
    .width(width)
    .height(height)
    .fovy(fovy)
    .near(near)
    .far(far)
    .step(step)
    .call())
}

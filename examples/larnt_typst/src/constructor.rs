use serde::{Deserialize, Serialize};

struct BilinearGrid {
    width: usize,
    height: usize,
    data: Vec<f64>,
    x_origin: f64,
    y_origin: f64,
    x_scale: f64,
    y_scale: f64,
}

impl BilinearGrid {
    fn new(
        width: usize,
        height: usize,
        data: Vec<f64>,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        Self {
            width,
            height,
            data,
            x_origin: x_range.0,
            y_origin: y_range.0,
            x_scale: (width - 1) as f64 / (x_range.1 - x_range.0),
            y_scale: (height - 1) as f64 / (y_range.1 - y_range.0),
        }
    }

    #[inline(always)]
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    fn get(&self, x: f64, y: f64) -> f64 {
        let u = ((x - self.x_origin) * self.x_scale).clamp(0.0, (self.width - 1) as f64);
        let v = ((y - self.y_origin) * self.y_scale).clamp(0.0, (self.height - 1) as f64);

        let ix = u.floor() as usize;
        let iy = v.floor() as usize;
        let tx = u - ix as f64;
        let ty = v - iy as f64;

        let ix1 = (ix + 1).min(self.width - 1);
        let iy1 = (iy + 1).min(self.height - 1);

        let q00 = self.data[iy * self.width + ix];
        let q10 = self.data[iy * self.width + ix1];
        let q01 = self.data[iy1 * self.width + ix];
        let q11 = self.data[iy1 * self.width + ix1];

        let top = Self::lerp(q00, q10, tx);
        let bot = Self::lerp(q01, q11, tx);
        Self::lerp(top, bot, ty)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub enum Direction {
    Above,
    #[default]
    Below,
}

impl Direction {
    fn to_direction(self) -> larnt::Direction {
        match self {
            Direction::Above => larnt::Direction::Above,
            Direction::Below => larnt::Direction::Below,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum FunctionTexture {
    Grid(f64),
    Swirl,
    Spiral,
}

impl Default for FunctionTexture {
    fn default() -> Self {
        FunctionTexture::Grid(1.0 / 8.0)
    }
}

impl FunctionTexture {
    fn to_texture(self) -> larnt::FunctionTexture {
        match self {
            FunctionTexture::Grid(grid_size) => larnt::FunctionTexture::Grid(grid_size),
            FunctionTexture::Swirl => larnt::FunctionTexture::Swirl,
            FunctionTexture::Spiral => larnt::FunctionTexture::Spiral,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub enum MeshTexture {
    #[default]
    Triangles,
    Polygonal,
    Silhouette(f64),
}

impl MeshTexture {
    fn to_texture(self) -> larnt::MeshTexture {
        match self {
            MeshTexture::Triangles => larnt::MeshTexture::Triangles,
            MeshTexture::Polygonal => larnt::MeshTexture::Polygonal,
            MeshTexture::Silhouette(cos_theta) => larnt::MeshTexture::Silhouette(cos_theta),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ParametricSurfaceTexture {
    Grid(f64),
    Triangles,
    Polygonal,
    Silhouette(f64),
}

impl Default for ParametricSurfaceTexture {
    fn default() -> Self {
        ParametricSurfaceTexture::Grid(1.0 / 8.0)
    }
}

impl ParametricSurfaceTexture {
    fn to_texture(self) -> Option<larnt::MeshTexture> {
        match self {
            ParametricSurfaceTexture::Grid(_) => None,
            ParametricSurfaceTexture::Triangles => Some(larnt::MeshTexture::Triangles),
            ParametricSurfaceTexture::Polygonal => Some(larnt::MeshTexture::Polygonal),
            ParametricSurfaceTexture::Silhouette(cos_theta) => {
                Some(larnt::MeshTexture::Silhouette(cos_theta))
            }
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
    Function {
        samples: Vec<Vec<f64>>,
        bbox: ([f64; 3], [f64; 3]),
        direction: Direction,
        texture: FunctionTexture,
        step: f64,
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
            LnShape::Function {
                samples,
                bbox,
                direction,
                texture,
                step,
            } => {
                let height = samples.len();
                let width = samples.first().map_or(0, Vec::len);
                if width < 2 || height < 2 {
                    return Err("Function samples must be at least 2x2".to_string());
                }
                if samples.iter().any(|row| row.len() != width) {
                    return Err("Function samples must have consistent row lengths".to_string());
                }

                let grid = BilinearGrid::new(
                    width,
                    height,
                    samples.into_iter().flatten().collect(),
                    (bbox.0[0], bbox.1[0]),
                    (bbox.0[1], bbox.1[1]),
                );
                let bx = larnt::BBox::new(
                    larnt::Vector::new(bbox.0[0], bbox.0[1], bbox.0[2]),
                    larnt::Vector::new(bbox.1[0], bbox.1[1], bbox.1[2]),
                );
                let func = move |x, y| grid.get(x, y);
                larnt::Primitive::Dynamic(Box::new(
                    larnt::Function::builder(func, bx)
                        .direction(direction.to_direction())
                        .texture(texture.to_texture())
                        .step(step)
                        .build(),
                ))
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
                    LnShape::Transformation {
                        shape: shape_inner,
                        matrix: Matrix::from_raw(matrix.to_matrix().mul(&matrix_inner.to_matrix())),
                    }
                    .to_shape()?
                } else {
                    larnt::TransformedShape::new(shape.to_shape()?, matrix.to_matrix()).into()
                }
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
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

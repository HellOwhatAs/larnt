#![doc = include_str!("../README.md")]

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
pub mod parametric;
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

pub use axis::Axis;
pub use bounding_box::BBox;
pub use cone::{Cone, ConeTexture, new_transformed_cone};
pub use csg::{BooleanShape, Op, new_difference, new_intersection};
pub use cube::{Cube, CubeTexture};
pub use cylinder::{Cylinder, CylinderTexture, new_transformed_cylinder};
pub use filter::{ClipFilter, Filter};
pub use function::{Direction, Function, FunctionTexture};
pub use hit::Hit;
pub use matrix::Matrix;
pub use mesh::Mesh;
pub use obj::load_obj;
pub use parametric::ParametricSurface;
pub use path::{NewPath, Paths};
pub use plane::Plane;
pub use ray::Ray;
pub use scene::render;
pub use shape::{EmptyShape, RenderArgs, Shape, TransformedShape};
pub use sphere::{Sphere, SphereTexture, lat_lng_to_xyz};
pub use stl::{load_binary_stl, load_stl, save_binary_stl};
pub use tree::Tree;
pub use triangle::Triangle;
pub use util::{degrees, median, radians};
pub use vector::Vector;

use derive_more::From;

#[derive(From)]
pub enum Primitive {
    EmptyShape(EmptyShape),
    Cone(Cone),
    Cube(Cube),
    Cylinder(Cylinder),
    Sphere(Sphere),
    Triangle(Box<Triangle>),
    Mesh(Box<Mesh>),
    ParametricSurface(Box<ParametricSurface>),
    TransformedShape(Box<TransformedShape<Self>>),
    BooleanShape(BooleanShape<Self>),
    Dynamic(Box<dyn Shape + Send + Sync>),
}

#[macro_export]
macro_rules! impl_shape_for_enum {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        impl Shape for $enum_name {
            #[inline(always)]
            fn bounding_box(&self) -> BBox { match self { $( $enum_name::$variant(inner) => inner.bounding_box(), )* } }

            #[inline(always)]
            fn contains(&self, v: Vector, f: f64) -> bool { match self { $( $enum_name::$variant(inner) => inner.contains(v, f), )* } }

            #[inline(always)]
            fn intersect(&self, r: Ray) -> Hit { match self { $( $enum_name::$variant(inner) => inner.intersect(r), )* } }

            #[inline(always)]
            fn paths(&self, args: &RenderArgs) -> Paths { match self { $( $enum_name::$variant(inner) => inner.paths(args), )* } }
        }
    };
}
impl_shape_for_enum!(Primitive {
    EmptyShape,
    Cone,
    Cube,
    Cylinder,
    Sphere,
    Triangle,
    Mesh,
    ParametricSurface,
    TransformedShape,
    BooleanShape,
    Dynamic,
});

#[macro_export]
macro_rules! impl_from_boxed_variant_for_enum {
    ($enum_name:ident {
        $(
            $variant:ident $( < $($generic:ty),+ $(,)? > )?
        ),* $(,)?
    }) => {
        $(
            impl From<$variant $( < $($generic),+ > )?> for $enum_name {
                fn from(val: $variant $( < $($generic),+ > )?) -> Self {
                    $enum_name::$variant(Box::new(val))
                }
            }
        )*
    };
}
impl_from_boxed_variant_for_enum!(Primitive {
    Triangle,
    Mesh,
    ParametricSurface,
    TransformedShape<Self>,
});

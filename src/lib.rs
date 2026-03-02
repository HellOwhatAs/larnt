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
pub use mesh::{Mesh, MeshTexture};
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

impl_from_for_enum!(Primitive {
    EmptyShape,
    Cone,
    Cube,
    Cylinder,
    Sphere,
    Triangle(Box<Triangle>),
    Triangle(Triangle => Box::new),
    Mesh(Box<Mesh>),
    Mesh(Mesh => Box::new),
    ParametricSurface(Box<ParametricSurface>),
    ParametricSurface(ParametricSurface => Box::new),
    TransformedShape(Box<TransformedShape<Self>>),
    TransformedShape(TransformedShape<Self> => Box::new),
    BooleanShape(BooleanShape<Self>),
    Dynamic(Box<dyn Shape + Send + Sync>),
});

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
            fn paths(&self, args: &RenderArgs) -> Paths<Vector> { match self { $( $enum_name::$variant(inner) => inner.paths(args), )* } }
        }
    };
}

#[macro_export]
macro_rules! impl_from_for_enum {
    // ==========================================
    // 0. Public Entry Point
    // ==========================================
    (
        $enum_name:ident {
            $($body:tt)*
        }
    ) => {
        impl_from_for_enum!(@parse_level $enum_name ; $($body)*);
    };

    // ==========================================
    // 1. @parse_level: Process direct child nodes at the top level
    // ==========================================

    // Case 1: With a custom mapping function (mapper), and has nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident ( $type:ty => $mapper:expr ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$variant(($mapper)(val)) }
        }
        impl_from_for_enum!(@flatten_map $enum_name, $variant, $type, $mapper ; $($children)*);
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Case 2: With a custom mapping function (mapper), no nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident ( $type:ty => $mapper:expr ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$variant(($mapper)(val)) }
        }
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Case 3: Explicitly declared type (e.g., A(A<usize>)), and has nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident ( $type:ty ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$variant(val) }
        }
        impl_from_for_enum!(@flatten $enum_name, $variant, $type ; $($children)*);
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Case 4: Implicit type (same as variant name), and has nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$variant> for $enum_name {
            fn from(val: $variant) -> Self { $enum_name::$variant(val) }
        }
        impl_from_for_enum!(@flatten $enum_name, $variant, $variant ; $($children)*);
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Case 5: Explicitly declared type, no nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident ( $type:ty ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$variant(val) }
        }
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Case 6: Implicit type, no nested child nodes
    (@parse_level $enum_name:ident ; $variant:ident $(, $($rest:tt)*)? ) => {
        impl From<$variant> for $enum_name {
            fn from(val: $variant) -> Self { $enum_name::$variant(val) }
        }
        $( impl_from_for_enum!(@parse_level $enum_name ; $($rest)*); )?
    };

    // Base case / Termination condition
    (@parse_level $enum_name:ident $(;)? ) => {};

    // ==========================================
    // 2. @flatten_map: Recursive flattening with mapper
    // ==========================================

    // Case 1: Grandchild node has its own mapper and is nested (top-level mapper takes precedence)
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident ( $type:ty => $_mapper:expr ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($children)*);
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Case 2: Grandchild node has its own mapper, no nesting
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident ( $type:ty => $_mapper:expr ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Case 3: Grandchild node with explicit type, has nested child nodes
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident ( $type:ty ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($children)*);
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Case 4: Grandchild node with implicit type, has nested child nodes
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$sub_var> for $enum_name {
            fn from(val: $sub_var) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($children)*);
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Case 5: Grandchild node with explicit type, no nesting
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident ( $type:ty ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Case 6: Grandchild node with implicit type, no nesting
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr ; $sub_var:ident $(, $($rest:tt)*)? ) => {
        impl From<$sub_var> for $enum_name {
            fn from(val: $sub_var) -> Self { $enum_name::$top_var(($mapper)(<$top_ty>::from(val))) }
        }
        $( impl_from_for_enum!(@flatten_map $enum_name, $top_var, $top_ty, $mapper ; $($rest)*); )?
    };

    // Base case / Termination condition
    (@flatten_map $enum_name:ident, $top_var:ident, $top_ty:ty, $mapper:expr $(;)? ) => {};

    // ==========================================
    // 3. @flatten: Normal recursive flattening without mapper
    // ==========================================

    // Case 1: Grandchild node has mapper, has nested child nodes
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident ( $type:ty => $_mapper:expr ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($children)*);
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Case 2: Grandchild node has mapper, no nesting
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident ( $type:ty => $_mapper:expr ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Case 3: Grandchild node with explicit type, has nested child nodes
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident ( $type:ty ) { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($children)*);
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Case 4: Grandchild node with implicit type, has nested child nodes
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident { $($children:tt)* } $(, $($rest:tt)*)? ) => {
        impl From<$sub_var> for $enum_name {
            fn from(val: $sub_var) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($children)*);
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Case 5: Grandchild node with explicit type, no nesting
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident ( $type:ty ) $(, $($rest:tt)*)? ) => {
        impl From<$type> for $enum_name {
            fn from(val: $type) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Case 6: Grandchild node with implicit type, no nesting
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty ; $sub_var:ident $(, $($rest:tt)*)? ) => {
        impl From<$sub_var> for $enum_name {
            fn from(val: $sub_var) -> Self { $enum_name::$top_var(<$top_ty>::from(val)) }
        }
        $( impl_from_for_enum!(@flatten $enum_name, $top_var, $top_ty ; $($rest)*); )?
    };

    // Base case / Termination condition
    (@flatten $enum_name:ident, $top_var:ident, $top_ty:ty $(;)? ) => {};
}

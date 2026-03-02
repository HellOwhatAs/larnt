use larnt::{
    BBox, Cube, Hit, Matrix, Paths, Primitive, Ray, RenderArgs, Shape, TransformedShape, Vector,
    impl_from_for_enum, impl_shape_for_enum, render,
};

#[derive(Debug, Clone)]
struct StripedCube {
    cube: Cube,
    stripes: i32,
}

impl Shape for StripedCube {
    fn bounding_box(&self) -> BBox {
        self.cube.bounding_box()
    }

    fn contains(&self, v: Vector, f: f64) -> bool {
        self.cube.contains(v, f)
    }

    fn intersect(&self, r: Ray) -> Hit {
        self.cube.intersect(r)
    }

    fn paths(&self, args: &RenderArgs) -> Paths<Vector> {
        let mut paths = self.cube.paths(args);
        let (x1, y1, z1) = (self.cube.min.x, self.cube.min.y, self.cube.min.z);
        let (x2, y2, z2) = (self.cube.max.x, self.cube.max.y, self.cube.max.z);

        for i in 0..=self.stripes {
            let p = i as f64 / self.stripes as f64;
            let x = x1 + (x2 - x1) * p;
            let y = y1 + (y2 - y1) * p;
            paths
                .new_path()
                .extend([Vector::new(x, y1, z1), Vector::new(x, y1, z2)]);
            paths
                .new_path()
                .extend([Vector::new(x, y2, z1), Vector::new(x, y2, z2)]);
            paths
                .new_path()
                .extend([Vector::new(x1, y, z1), Vector::new(x1, y, z2)]);
            paths
                .new_path()
                .extend([Vector::new(x2, y, z1), Vector::new(x2, y, z2)]);
        }
        paths
    }
}

fn main() {
    let striped_cube = StripedCube {
        cube: Cube::builder(Vector::default(), Vector::new(1., 1., 1.)).build(),
        stripes: 20,
    };
    let cube = Cube::builder(Vector::new(-1., 0., 0.), Vector::new(0., 1., 1.2)).build();

    // dynamic dispatch using `Primitive::Dynamic`.
    let dynamic_dispatch = {
        let striped_cube: Primitive =
            (Box::new(striped_cube.clone()) as Box<dyn Shape + Send + Sync>).into();
        assert!(matches!(striped_cube, Primitive::Dynamic(_)));
        let transformed_striped_cube: TransformedShape<Primitive> = TransformedShape::new(
            striped_cube.into(),
            Matrix::rotate(Vector::new(1., 0., 0.), std::f64::consts::PI / 4.),
        );
        render::<Primitive>(vec![transformed_striped_cube.into(), cube.clone().into()])
            .eye(Vector::new(2., 3., 4.))
            .call()
    };

    // static dispatch using a new enum `MyPrimitive`.
    let static_dispatch = {
        enum MyPrimitive {
            Primitive(Primitive),
            TransformedShape(Box<TransformedShape<Self>>),
            StripedCube(StripedCube),
        }
        impl_shape_for_enum!(MyPrimitive {
            Primitive,
            TransformedShape,
            StripedCube
        });
        impl_from_for_enum!(MyPrimitive {
            StripedCube,
            TransformedShape(Box<TransformedShape<Self>>),
            TransformedShape(TransformedShape<Self> => Box::new),
            Primitive { Cube(larnt::Cube) }
        });
        let transformed_striped_cube: TransformedShape<MyPrimitive> = TransformedShape::new(
            striped_cube.into(),
            Matrix::rotate(Vector::new(1., 0., 0.), std::f64::consts::PI / 4.),
        );
        render::<MyPrimitive>(vec![transformed_striped_cube.into(), cube.into()])
            .eye(Vector::new(2., 3., 4.))
            .call()
    };

    assert!(
        dynamic_dispatch
            .iter_paths()
            .zip(static_dispatch.iter_paths())
            .all(|(a, b)| a == b)
    );
}

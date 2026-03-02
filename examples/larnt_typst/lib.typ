#let larnt = plugin("./larnt_typst_plugin.wasm")
#import "./texture.typ"

/// The cone shape.
/// Can be warped with `outline()` to create outline cone.
///
/// ```example
/// #image(render(
///   eye: (1.2, -3., 2.),
///   center: (1.2, 0., 0.2),
///   height: 512.,
///   cone(1., (0., 0., 0.), (0., 0., 1.)),
///   cone(1., (2.4, 0., 0.), (2.4, 0., 1.), texture: texture.striped(15)),
/// ))
/// ```
///
/// -> shape
#let cone(
  /// The radius of the cone.
  /// -> float
  radius,
  /// The starting point of the cone's axis, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v0,
  /// The ending point of the cone's axis, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v1,
  /// The texture pattern applied to the cone's surface. Can be `outline` or `striped`.
  /// -> texture
  texture: texture.outline(),
) = {
  assert(
    type(radius) == float and (v0, v1).all(v => type(v) == array and v.len() == 3 and v.all(i => type(i) == float)),
    message: "cone(radius, v0, v1) expects `radius` be a float and `v0`, `v1` be arrays of 3 floats",
  )
  assert(
    texture == "Outline" or (type(texture) == dictionary and "Striped" in texture),
    message: "cone(...) texture must be either `outline()` or `striped(num)`",
  )
  return (
    Cone: (
      radius: radius,
      v0: v0,
      v1: v1,
      texture: texture,
    ),
  )
}

/// The cube shape.
///
/// ```example
/// #image(render(
///   eye: (1.25, 2.5, 2.0),
///   center: (1.25, -1., -0.6),
///   height: 460.,
///   cube((0., 0., 0.), (1., 1., 1.)),
///   cube((1.5, 0., 0.), (2.5, 1., 1.), texture: texture.striped(24)),
/// ))
/// ```
///
/// -> shape
#let cube(
  /// The minimum corner of the cube, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  min,
  /// The maximum corner of the cube, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  max,
  /// The texture pattern applied to the cube's surface. Can be `vanilla` or `striped`.
  /// -> texture
  texture: texture.vanilla(),
) = {
  assert(
    (min, max).all(x => type(x) == array and x.len() == 3 and x.all(i => type(i) == float)),
    message: "cube(min, max, ..) expects two array of 3 floats",
  )
  assert(
    texture == "Vanilla" or (type(texture) == dictionary and "Striped" in texture),
    message: "cube(...) texture must be either `vanilla()` or `striped(num)`",
  )
  return (
    Cube: (
      min: min,
      max: max,
      texture: texture,
    ),
  )
}

/// The cylinder shape. Can be warped with `outline()` to create outline cylinder.
///
/// ```example
/// #image(render(
///   eye: (1.2, -3., 3.2),
///   center: (1.2, 0., .2),
///   height: 600.,
///   cylinder(0.7, (0., 0., 0.), (0., 0., 1.)),
///   cylinder(0.7, (2.4, 0., 0.), (2.4, 0., 1.), texture: texture.striped(64)),
/// ))
/// ```
///
/// -> shape
#let cylinder(
  /// The radius of the cylinder.
  /// -> float
  radius,
  /// The starting point of the cylinder's axis, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v0,
  /// The ending point of the cylinder's axis, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v1,
  /// The texture pattern applied to the cylinder's surface. Can be `outline` or `striped`.
  /// -> texture
  texture: texture.outline(),
) = {
  assert(
    type(radius) == float and (v0, v1).all(v => type(v) == array and v.len() == 3 and v.all(i => type(i) == float)),
    message: "cylinder(radius, v0, v1) expects `radius` be a float and `v0`, `v1` be arrays of 3 floats",
  )
  assert(
    texture == "Outline" or (type(texture) == dictionary and "Striped" in texture),
    message: "cylinder(...) texture must be either `outline()` or `striped(num)`",
  )
  return (
    Cylinder: (
      radius: radius,
      v0: v0,
      v1: v1,
      texture: texture,
    ),
  )
}

/// The sphere shape. Can be warped with `outline()` to create outline sphere.
///
/// ```example
/// #image(render(
///   height: 512.,
///   fovy: 30.,
///   sphere((0., 0., 0.), 1.0),
///   sphere((0., -2.2, 0.), 1.0, texture: texture.lat_lng()),
///   sphere((2.2, 0., 0.), 1.0, texture: texture.random_circles(42)),
///   sphere((0., 2.2, 0.), 1.0, texture: texture.random_equators(42)),
///   sphere((-2.2, 0., 0.), 1.0, texture: texture.random_fuzz(42, num: 5000)),
/// ))
/// ```
/// -> shape
#let sphere(
  /// The center of the sphere, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  center,
  /// The radius of the sphere.
  /// -> float
  radius,
  /// The texture pattern applied to the sphere's surface. Can be one of `outline`, `lat_lng`, `random_equators`, `random_fuzz`, or `random_circles`.
  /// -> texture
  texture: texture.outline(),
) = {
  assert(
    type(center) == array and center.len() == 3 and center.all(i => type(i) == float) and type(radius) == float,
    message: "sphere(...) expects `center` be an array of 3 floats, `radius` be a float",
  )
  assert(
    texture == "Outline"
      or (
        type(texture) == dictionary
          and (
            "LatLng" in texture or "RandomEquators" in texture or "RandomFuzz" in texture or "RandomCircles" in texture
          )
      ),
    message: "sphere(...) texture must be either `outline()`, `lat_lng()`, `random_equators(seed, n)`, `random_fuzz(seed, num, scale)`, or `random_circles(seed, num)`",
  )
  return (
    Sphere: (
      center: center,
      radius: radius,
      texture: texture,
    ),
  )
}

/// The triangle shape.
///
/// ```example
/// #image(render(
///   eye: (2., 2., 2.),
///   center: (0., 0., 0.),
///   height: 512.,
///   triangle((0., 1., 0.), (1., 0., 0.), (0., 0., 0.)),
///   triangle((0., 1., 0.), (1., 0., 0.), (1., 1., 0.)),
/// ))
/// ```
///
/// -> shape
#let triangle(
  /// The first vertex of the triangle, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v1,
  /// The second vertex of the triangle, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v2,
  /// The third vertex of the triangle, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  v3,
) = {
  assert(
    (v1, v2, v3).all(v => type(v) == array and v.len() == 3 and v.all(i => type(i) == float)),
    message: "triangle(v1, v2, v3) expects three arrays of 3 floats",
  )
  return (
    Triangle: (
      v1: v1,
      v2: v2,
      v3: v3,
    ),
  )
}

/// The mesh shape composed of triangles.
/// The edge lines of adjacent and coplanar triangles will hide to create smooth surface.
///
/// ```example
/// #image(render(
///   eye: (2., 3., 2.),
///   fovy: 30.,
///   height: 360.,
///   mesh(
///     ((0., 0., 0.), (1., 0., 0.), (0., 1., 0.), (1., 1., 0.)),
///     (0, 1, 2, 3, 2, 1),
///   ),
/// ))
/// ```
///
/// -> shape
#let mesh(
  /// An array of vertices if triangles.
  /// -> array
  vertices,
  /// An array of indices defining the triangle vertices. Each group of three consecutive indices represents one triangle, referencing the corresponding vertices in the `vertices` array.
  /// -> array
  triangles,
  /// An array of normal flipped triangle index pairs.
  /// Useful for non-orientable surface such as Möbius strip and Klein bottle.
  /// -> array
  flipped_triangles: (),
  /// Texture of the Mesh.  Can be one of `triangles`, `polygonal`, or `silhouette`.
  /// -> texture
  texture: texture.triangles(),
) = {
  assert(
    type(vertices) == array and type(triangles) == array and type(flipped_triangles) == array,
    message: "mesh(..) expects three array",
  )
  return (
    Mesh: (
      vertices: vertices,
      triangles: triangles,
      flipped_triangles: flipped_triangles,
      texture: texture,
    ),
  )
}

/// The parametric surface defined by a function of (u, v) => (x, y, z).
/// It generates a sampled mesh across the parameter grid, applying a custom grid texture for rendering.
///
/// ```example
/// #import "@preview/lilaq:0.5.0": linspace
/// #image(render(
///   eye: (3., 3., 3.),
///   height: 600.,
///   surface(
///     linspace(0, calc.pi * 2, num: 64),
///     linspace(0.0, calc.pi * 2, num: 32),
///     (u, v) => {
///       let x = (1.5 + 0.5 * calc.cos(v)) * calc.cos(u);
///       let y = (1.5 + 0.5 * calc.cos(v)) * calc.sin(u);
///       let z = 0.5 * calc.sin(v);
///       (x, y, z)
///     }
///   ),
/// ))
/// ```
///
/// -> shape
#let surface(
  /// The u parameter samples. Given as an array of floats.
  /// -> array
  u,
  /// The v parameter samples. Given as an array of floats.
  /// -> array
  v,
  /// The function that defines the surface, given as a function that takes two floats (u and v) and returns an array of three floats representing the x, y, and z coordinates of the surface point corresponding to those parameters.
  func,
  /// The texture pattern of the surface. Can be one of `grid`, `triangles`, `polygonal`, or `silhouette`.
  /// -> texture
  texture: texture.grid(),
) = {
  (
    ParametricSurface: (
      samples: u.map(u => v.map(v => func(u, v))).join(),
      u_steps: u.len() - 1,
      v_steps: v.len() - 1,
      texture: texture,
    ),
  )
}

/// The difference operation for shapes.
/// Can be used to create complex shapes by subtracting multiple shapes from a base shape.
///
/// ```example
/// #image(render(
///   eye: (3., 2., 2.),
///   center: (0., 0.1, 0.),
///   fovy: 30.,
///   difference(
///     cube((0., 0., 0.), (1., 1., 1.), texture: texture.striped(15)),
///     sphere((1., 1., 0.5), 0.5, texture: texture.lat_lng()),
///     sphere((0., 1., 0.5), 0.5, texture: texture.lat_lng()),
///     sphere((0., 0., 0.5), 0.5, texture: texture.lat_lng()),
///     sphere((1., 0., 0.5), 0.5, texture: texture.lat_lng()),
///   ),
/// ))
/// ```
///
/// -> shape
#let difference(
  /// The shapes to be subtracted.
  /// The first shape is the base shape, and the subsequent shapes are subtracted from it.
  /// At least two shapes are required.
  /// -> shape
  ..shapes,
) = {
  let shapes = shapes.pos()
  assert(
    shapes.len() >= 2 and shapes.all(s => type(s) == dictionary),
    message: "difference(...) expects two or more shape arguments",
  )
  return (
    Difference: shapes,
  )
}

/// The intersection operation for shapes.
/// Can be used to create complex shapes by intersecting multiple shapes.
///
/// ```example
/// #image(render(
///   eye: (3., 2., 2.),
///   center: (0., 0., 0.5),
///   fovy: 20.,
///   intersection(
///     sphere((0., 0., 0.5), 0.6, texture: texture.lat_lng(n: 10)),
///     cube((-0.5, -0.5, 0.), (.5, 0.5, 1.0), texture: texture.striped(32)),
///   ),
/// ))
/// ```
///
/// -> shape
#let intersection(
  /// The shapes to be intersected.
  /// At least two shapes are required.
  /// -> shape
  ..shapes,
) = {
  let shapes = shapes.pos()
  assert(
    shapes.len() >= 2 and shapes.all(s => type(s) == dictionary),
    message: "intersection(...) expects two or more shape arguments",
  )
  return (
    Intersection: shapes,
  )
}

/// Translates a shape by a given vector.
///
/// -> shape
#let translate(
  /// The shape to be translated.
  /// -> shape
  shape,
  /// The translation vector, given as an array of three floats representing the x, y, and z components.
  /// -> array
  v,
) = {
  assert(
    type(shape) == dictionary and type(v) == array and v.len() == 3 and v.all(i => type(i) == float),
    message: "translate(shape, v) expects a shape and an array of 3 floats",
  )
  return (
    Transformation: (
      shape: shape,
      matrix: (Translate: (v: v)),
    ),
  )
}

/// Rotates a shape around a given axis by a specified angle.
///
/// -> shape
#let rotate(
  /// The shape to be rotated.
  /// -> shape
  shape,
  /// The rotation axis, given as an array of three floats representing the x, y, and z components.
  /// -> array
  v,
  /// The rotation angle in radians.
  /// -> float
  a,
) = {
  assert(
    type(shape) == dictionary
      and type(v) == array
      and v.len() == 3
      and v.all(i => type(i) == float)
      and type(a) == float,
    message: "rotate(shape, v, a) expects a shape, an array of 3 floats, and a float",
  )
  return (
    Transformation: (
      shape: shape,
      matrix: (
        Rotate: (
          v: v,
          a: a,
        ),
      ),
    ),
  )
}

/// Scales a shape by given factors along each axis.
///
/// -> shape
#let scale(
  /// The shape to be scaled.
  /// -> shape
  shape,
  /// The scaling factors along the x, y, and z axes, given as an array of three floats.
  /// -> array
  v,
) = {
  assert(
    type(shape) == dictionary and type(v) == array and v.len() == 3 and v.all(i => type(i) == float),
    message: "scale(shape, v) expects a shape and an array of 3 floats",
  )
  return (
    Transformation: (
      shape: shape,
      matrix: (Scale: (v: v)),
    ),
  )
}

/// Renders a 3D scene defined by the given shapes from a specified camera viewpoint.
/// Returns the rendered SVG as bytes.
///
/// ```example
/// #image(render(
///   eye: (2., 7., 5.),
///   center: (1.5, 2., 0.),
///   cube((0., 0., 0.), (1., 1., 1.)),
///   cube((1.5, 0., 0.), (2.5, 1., 1.), texture: texture.striped(8)),
///   sphere((0.5, 2., .5), 0.5),
///   sphere((2., 2., .5), 0.5, texture: texture.random_circles(42)),
///   sphere((0.5, 3.5, .5), 0.5, texture: texture.random_equators(42)),
///   sphere((2., 3.5, .5), 0.5, texture: texture.lat_lng()),
///   sphere((3.5, 3.5, .5), 0.5, texture: texture.random_fuzz(42)),
///   cone(.5, (-1., .5, 0.), (-1., .5, 1.)),
///   cone(.5, (-1., 2., 0.), (-1., 2., 1.), texture: texture.striped(15)),
///   cylinder(.5, (3.5, .5, 0.), (3.5, .5, 1.)),
///   cylinder(.5, (3.5, 2., 0.), (3.5, 2., 1.), texture: texture.striped(32))))
/// ```
///
/// -> bytes
#let render(
  /// The position of the camera in 3D space, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  eye: (5.0, 5.0, 5.0),
  /// The point in 3D space that the camera is looking at, given as an array of three floats representing the x, y, and z coordinates.
  /// -> array
  center: (0.0, 0.0, 0.0),
  /// The up direction for the camera, given as an array of three floats.
  /// -> array
  up: (0.0, 0.0, 1.0),
  /// The width of the output image.
  /// -> float
  width: 720.0,
  /// The height of the output image.
  /// -> float
  height: 720.0,
  /// Field of view of the camera.
  /// -> float
  fovy: 50.0,
  /// The near clipping plane distance.
  /// -> float
  near: 0.1,
  /// The far clipping plane distance.
  /// -> float
  far: 1000.0,
  /// Controls rendering precision, representing the maximum error distance (in screen-space units)
  /// between a line and its true position. Smaller step size results in more accurate rendering but longer rendering time.
  /// Because the error is measured in screen space, the perspective projection naturally provides an automatic
  /// Level of Detail (LOD) effect.
  ///
  /// ```example
  /// #let shapes = (
  ///   cube((0., 0., 0.), (1., 1., 1.)),
  ///   cube((1.5, 0., 0.), (2.5, 1., 1.), texture: texture.striped(8)),
  ///   sphere((0.5, 2., .5), 0.5),
  ///   sphere((2., 2., .5), 0.5, texture: texture.random_circles(42)),
  ///   sphere((0.5, 3.5, .5), 0.5, texture: texture.random_equators(42)),
  ///   sphere((2., 3.5, .5), 0.5, texture: texture.lat_lng()),
  ///   sphere((3.5, 3.5, .5), 0.5, texture: texture.random_fuzz(42)),
  ///   cone(.5, (-1., .5, 0.), (-1., .5, 1.)),
  ///   cone(.5, (-1., 2., 0.), (-1., 2., 1.), texture: texture.striped(15)),
  ///   cylinder(.5, (3.5, .5, 0.), (3.5, .5, 1.)),
  ///   cylinder(.5, (3.5, 2., 0.), (3.5, 2., 1.), texture: texture.striped(32)),
  /// )
  /// #grid(
  ///   ..(10.0, 30.0, 100.0).map(step => image(render(
  ///     eye: (2., 7., 5.),
  ///     center: (1.5, 2., 0.),
  ///     step: step,
  ///     fovy: 32.0,
  ///     height: 480.0,
  ///     ..shapes,
  ///   )))
  /// )
  /// ```
  ///
  /// -> float
  step: 1.0,
  /// The output format of the rendered image. Supports `"svg"` or `"png"` or `("png": ("linewidth": <float>))`.
  /// -> str
  format: "svg",
  /// The 3D shapes to be rendered in the scene.
  /// -> shape
  ..shapes,
) = {
  larnt.render(
    cbor.encode((
      eye: eye,
      center: center,
      up: up,
      width: width,
      height: height,
      fovy: fovy,
      near: near,
      far: far,
      step: step,
      format: if format == "png" { ("png": ("linewidth": 1.0)) } else { format },
    )),
    cbor.encode(shapes.pos()),
  )
}


#import "@preview/lilaq:0.5.0": linspace
#set page(width: auto, height: auto, margin: 0pt)
#image(render(
  eye: (3., -5., 2.),
  surface(
    linspace(0., 2. * calc.pi),
    linspace(-0.5, 0.5),
    (u, v) => {
      let x = (2.0 + (v / 2.0) * calc.cos(u / 2.0)) * calc.cos(u)
      let y = (2.0 + (v / 2.0) * calc.cos(u / 2.0)) * calc.sin(u)
      let z = (v / 2.0) * calc.sin(u / 2.0)
      (x, y, z)
    },
    texture: "Silhouette",
  ),
))

#set page(margin: 0pt, height: auto)
#let ln = plugin("./ln_typst_plugin.wasm")
#import "@preview/suiji:0.5.1"
#import "@preview/eqalc:0.1.4"

#let cone(radius, v0, v1) = {
  assert(
    type(radius) == float and (v0, v1).all(v => type(v) == array and v.len() == 3 and v.all(i => type(i) == float)),
    message: "cone(radius, height) expects `radius` be a float and `v0`, `v1` be arrays of 3 floats",
  )
  return (
    Cone: (
      radius: radius,
      v0: v0,
      v1: v1,
    ),
  )
}

#let cube(min, max, texture: "Vanilla", stripes: 0) = {
  assert(
    (min, max).all(x => type(x) == array and x.len() == 3 and x.all(i => type(i) == float)),
    message: "cube(min, max) expects two array of 3 floats",
  )
  assert(texture in ("Vanilla", "Stripes"), message: "cube(...) texture must be one of Vanilla, Stripes")
  return (
    Cube: (
      min: min,
      max: max,
      texture: texture,
      stripes: stripes,
    ),
  )
}

#let cylinder(radius, v0, v1) = {
  assert(
    type(radius) == float and (v0, v1).all(v => type(v) == array and v.len() == 3 and v.all(i => type(i) == float)),
    message: "cylinder(radius, v0, v1) expects `radius` be a float and `v0`, `v1` be arrays of 3 floats",
  )
  return (
    Cylinder: (
      radius: radius,
      v0: v0,
      v1: v1,
    ),
  )
}

#let sphere(center, radius, texture: "LatLng", seed: 0) = {
  assert(
    type(center) == array
      and center.len() == 3
      and center.all(i => type(i) == float)
      and type(radius) == float
      and type(texture) == str
      and type(seed) == int,
    message: "sphere(...) expects `center` be an array of 3 floats, `radius` be a float, `texture` be a string, and `seed` be an integer",
  )
  assert(
    texture in ("LatLng", "RandomEquators", "RandomDots", "RandomCircles"),
    message: "sphere(...) texture must be one of LatLng, RandomEquators, RandomDots, RandomCircles",
  )
  return (
    Sphere: (
      center: center,
      radius: radius,
      texture: texture,
      seed: seed,
    ),
  )
}

#let func(func, min, max, direction: "Below", texture: "Grid") = {
  assert(
    ((type(func) == content and func.func() == math.equation) or type(func) == str)
      and type(min) == array
      and min.len() == 3
      and min.all(i => type(i) == float)
      and type(max) == array
      and max.len() == 3
      and max.all(i => type(i) == float)
      and type(direction) == str
      and type(texture) == str,
    message: "func(...) expects `func` be either an equation content or a string, `min` and `max` be arrays of 3 floats, `direction` and `texture` be strings",
  )
  assert(
    direction in ("Below", "Above"),
    message: "func(...) direction must be one of Below, Above",
  )
  assert(
    texture in ("Grid", "Spiral", "Swirl"),
    message: "func(...) texture must be one of Grid, Spiral, or Swirl",
  )
  return (
    Function: (
      func: if type(func) == content and func.func() == math.equation {
        eqalc
          .math-to-str(func)
          .replace("calc.div-euclid", "div_euclid")
          .replace("calc.rem-euclid", "rem_euclid")
          .replace("calc.", "")
      } else {
        func
      },
      bbox: (min, max),
      direction: direction,
      texture: texture,
    ),
  )
}

#let triangle(v1, v2, v3) = {
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

#let mesh(triangles) = {
  assert(
    type(triangles) == array
      and triangles.all(t => (
        type(t) == dictionary and "Triangle" in t
      )),
    message: "mesh(triangles) expects an array of triangles",
  )
  return (
    Mesh: triangles,
  )
}

#let outline(shape) = {
  assert(
    type(shape) == dictionary and ("Cone", "Cylinder", "Sphere").any(s => s in shape),
    message: "outline(shape) expects cone, cylinder, or sphere shape",
  )
  return (
    Outline: shape,
  )
}

#let difference(..shapes) = {
  let shapes = shapes.pos()
  assert(
    shapes.len() >= 2 and shapes.all(s => type(s) == dictionary),
    message: "difference(...) expects two or more shape arguments",
  )
  return (
    Difference: shapes,
  )
}

#let intersection(..shapes) = {
  let shapes = shapes.pos()
  assert(
    shapes.len() >= 2 and shapes.all(s => type(s) == dictionary),
    message: "intersection(...) expects two or more shape arguments",
  )
  return (
    Intersection: shapes,
  )
}

#let translate(shape, v) = {
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

#let rotate(shape, v, a) = {
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

#let scale(shape, v) = {
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

#let render(
  eye: (5.0, 5.0, 5.0),
  center: (0.0, 0.0, 0.0),
  up: (0.0, 0.0, 1.0),
  width: 1024.0,
  height: 1024.0,
  fovy: 50.0,
  near: 0.1,
  far: 100.0,
  step: 0.1,
  ..shapes,
) = {
  image(ln.render(
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
    )),
    cbor.encode(shapes.pos()),
  ))
}


// =============================================================
#let rng = suiji.gen-rng-f(42)

#render(
  eye: (2., 7., 5.),
  center: (1.5, 2., 0.),
  step: 0.01,
  cube((0., 0., 0.), (1., 1., 1.)),
  cube((1.5, 0., 0.), (2.5, 1., 1.), texture: "Stripes", stripes: 8),
  sphere((0.5, 2., .5), 0.5),
  sphere((2., 2., .5), 0.5, texture: "RandomCircles"),
  sphere((0.5, 3.5, .5), 0.5, texture: "RandomEquators"),
  outline(sphere((2., 3.5, .5), 0.5)),
  cone(0.5, (-1., 0.5, 0.), (-1., 0.5, 1.)),
  translate(outline(cone(0.5, (0., 0., 0.), (0., 0., 1.))), (-1., 2.0, 0.)),
  cylinder(0.5, (3.5, 0.5, 0.), (3.5, 0.5, 1.)),
  translate(outline(cylinder(0.5, (0., 0., 0.), (0., 0., 1.))), (3.5, 2.0, 0.)),
)

#{
  let cubes = ()
  for x in range(-2, 3) {
    for y in range(-2, 3) {
      let z
      (rng, z) = suiji.random-f(rng)
      cubes.push(cube(
        (x - 0.5, y - 0.5, z - 0.5),
        (x + 0.5, y + 0.5, z + 0.5),
        texture: "Stripes",
        stripes: 7,
      ))
    }
  }
  render(step: 0.05, ..cubes)
}

#render(
  eye: (3., 2.5, 2.0),
  center: (1., 0.0, 0.0),
  step: 0.05,
  difference(
    cube((0., 0., 0.), (1., 1., 1.), texture: "Stripes", stripes: 15),
    sphere((1., 1., 0.5), 0.5),
    sphere((0., 1., 0.5), 0.5),
    sphere((0., 0., 0.5), 0.5),
    sphere((1., 0., 0.5), 0.5),
  ),
  cube((0.3, 0.3, 0.), (.7, .7, 1.2), texture: "Stripes", stripes: 10),
  intersection(
    sphere((2.0, 0.25, 0.5), 0.6),
    cube((1.5, -0.25, 0.), (2.5, 0.75, 1.0), texture: "Stripes", stripes: 25),
  ),
)

#{
  let n = 8
  let shapes = ()
  for x in range(-n, n + 1) {
    for y in range(-n, n + 2) {
      let seed
      (rng, seed) = suiji.integers-f(rng)
      shapes.push(
        outline(
          sphere(
            (float(x), float(y), 0.),
            0.45,
            texture: "RandomCircles",
            seed: seed,
          ),
        ),
      )
    }
  }
  render(
    eye: (8.0, 8.0, 1.0),
    center: (0., 0., -4.25),
    ..shapes,
  )
}

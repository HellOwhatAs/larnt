#set page(margin: 0pt, height: auto)
#let ln = plugin("./ln_typst_plugin.wasm")
#import "@preview/eqalc:0.1.4"

#image(ln.render(
  cbor.encode((
    eye: (3.75, 1.75, 2.0),
    center: (0.0, 0.0, 0.0),
    up: (0.0, 0.0, 1.0),
    width: 1024.0,
    height: 1024.0,
    fovy: 50.0,
    near: 0.1,
    far: 30.0,
    step: 0.01,
  )),
  cbor.encode((
    (
      Difference: (
        (
          Cube: (
            min: (0.0, 0.0, 0.0),
            max: (1.0, 1.0, 1.0),
          ),
        ),
        (
          Sphere: (
            center: (1.1, 1.1, 1.0),
            radius: 0.6,
            texture: "LatLng",
          ),
        ),
      ),
    ),
    (
      Sphere: (
        center: (2.0, 0.25, 0.5),
        radius: 0.6,
        texture: "RandomCircles",
        seed: 42,
      ),
    ),
  )),
))


#image(ln.render(
  cbor.encode((
    eye: (3.75, 1.75, 5.0),
    center: (0.0, 0.0, 0.0),
    up: (0.0, 0.0, 1.0),
    width: 1024.0,
    height: 1024.0,
    fovy: 50.0,
    near: 0.1,
    far: 30.0,
    step: 1.,
  )),
  cbor.encode((
    (
      Function: (
        func: eqalc
          .math-to-str($x dot y$)
          .replace("calc.div-euclid", "div_euclid")
          .replace("calc.rem-euclid", "rem_euclid")
          .replace("calc.", ""),
        bbox: (
          (-1.0, -1.0, -1.0),
          (1.0, 1.0, 1.0),
        ),
        direction: "Below",
        texture: "Swirl",
      ),
    ),
    (
      Function: (
        func: eqalc
          .math-to-str($0$)
          .replace("calc.div-euclid", "div_euclid")
          .replace("calc.rem-euclid", "rem_euclid")
          .replace("calc.", ""),
        bbox: (
          (-1.0, -1.0, -1.0),
          (1.0, 1.0, 1.0),
        ),
        direction: "Below",
        texture: "Grid",
      ),
    ),
    (
      Sphere: (
        center: (2.0, 0.25, 0.5),
        radius: 0.6,
        texture: "RandomCircles",
        seed: 42,
      ),
    ),
  )),
))

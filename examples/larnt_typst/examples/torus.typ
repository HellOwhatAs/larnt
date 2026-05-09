#set page(margin: 0pt, height: auto)
#import "../lib.typ": *
#import "@preview/lilaq:0.5.0": linspace

#let radius = 1.5;
#let tube_radius = 0.5;
#let torus_func = (u, v) => {
  let x = (radius + tube_radius * calc.cos(v)) * calc.cos(u)
  let y = (radius + tube_radius * calc.cos(v)) * calc.sin(u)
  let z = tube_radius * calc.sin(v)
  (x, y, z)
};
#let twisted_func = (u, v) => {
  let k = 1.0
  let v_shifted = v + k * u

  let x = (radius + tube_radius * calc.cos(v_shifted)) * calc.cos(u)
  let y = (radius + tube_radius * calc.cos(v_shifted)) * calc.sin(u)
  let z = tube_radius * calc.sin(v_shifted)
  (x, y, z)
};
#let offset = radius + tube_radius;

#image(
  render(
    eye: (3., 3., 2.2),
    height: 200.0,
    width: 200.0,
    rotate(
      translate(
        surface(
          linspace(0, calc.pi * 2, num: 64),
          linspace(0, calc.pi * 2, num: 32),
          torus_func,
          texture: texture.grid(),
        ),
        (offset, 0., 0.),
      ),
      (1., 0., 0.),
      calc.pi / 2,
    ),
    surface(
      linspace(0, calc.pi * 2, num: 20),
      linspace(0, calc.pi * 2, num: 10),
      twisted_func,
      texture: texture.triangles(),
    ),
    rotate(
      translate(
        surface(
          linspace(0, calc.pi * 2),
          linspace(0, calc.pi * 2),
          torus_func,
          texture: texture.silhouette(),
        ),
        (-offset, 0., 0.),
      ),
      (1., 0., 0.),
      calc.pi / 2,
    ),
  ),
  width: 100%,
)

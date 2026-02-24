#set page(margin: 0pt, height: auto)
#import "../lib.typ": *
#import "@preview/lilaq:0.5.0": linspace

#image(render(
  eye: (3., 3., 3.),
  step: 10.0,
  surface(
    linspace(0, calc.pi * 2, num: 64),
    linspace(0.0, calc.pi * 2, num: 32),
    (u, v) => {
      let x = (1.5 + 0.5 * calc.cos(v)) * calc.cos(u);
      let y = (1.5 + 0.5 * calc.cos(v)) * calc.sin(u);
      let z = 0.5 * calc.sin(v);
      (x, y, z)
    }
  ),
), width: 100%)

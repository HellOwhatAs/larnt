#set page(height: auto, margin: 0pt)
#import "../lib.typ": *
#import "@preview/lilaq:0.5.0": linspace

#{
  let f(x, y) = (x, y, 2.0 * calc.sin(calc.sqrt(x * x + y * y)))
  image(
    render(
      eye: (75., 35., 50.),
      center: (5., 0., 0.),
      fovy: 32.,
      surface(linspace(-20.0, 20.0, num: 100), linspace(-20.0, 20.0, num: 100), f),
    ),
    width: 100%,
  )
}

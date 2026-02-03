#set page(height: auto, margin: 0pt)
#import "./lib.typ": *

#{
  let f(x, y) = 0.7 * calc.sin(calc.sqrt(20 * (x * x + y * y)))
  let (min, max) = ((-4., -4., -4.), (4., 4., 4.))
  render(
    eye: (7., 7., 7.),
    step: 0.01,
    func(f, min, max),
  )
}


#{
  let (min, max) = ((-1., -1., -1.), (1., 1., 1.))
  render(
    eye: (3., 0.5, 3.),
    step: 0.01,
    func((x, y) => x * y, min, max, texture: "Spiral"),
    func((x, y) => 0.0, min, max),
    sphere((0., -0.6, 0.), 0.25, texture: "RandomCircles"),
  )
}

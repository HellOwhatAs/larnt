// Takes about 1.5s to render.
#set page(height: auto, margin: 0pt)
#import "../lib.typ": *
#import "@preview/lilaq:0.5.0": linspace

#{
  let (min, max) = ((-3., -3., -4.), (3., 3., 2.))
  image(
    render(
      eye: (3., 0., 3.),
      center: (1.1, 0., 0.),
      surface(linspace(-3, 3), linspace(-3, 3), (x, y) => (x, y, -1 / (x * x + y * y)))
    ),
    width: 100%,
  )
}

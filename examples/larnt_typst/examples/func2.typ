// Takes about 1s to render.
#set page(height: auto, margin: 0pt)
#import "../lib.typ": *
#import "@preview/lilaq:0.5.0": linspace

#{
  let x = linspace(-1., 1.)
  let y = linspace(-1., 1.)
  image(
    render(
      eye: (3., 0.5, 3.),
      surface(x, y, (x, y) => (x, y, x * y), texture: texture.grid()),
      surface(x, y, (x, y) => (x, y, 0.), texture: texture.silhouette()),
      sphere((0., -0.6, 0.), 0.25, texture: texture.random_circles(42)),
    ),
    width: 100%,
  )
}

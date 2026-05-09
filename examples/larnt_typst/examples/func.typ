// Takes about 1.5s to render.
#set page(height: auto, margin: 0pt)
#import "../lib.typ": *

#{
  image(
    render(
      eye: (3., 0.5, 3.),
      fovy: 40.,
      func(
        (x, y) => x * y,
        (-1., -1., -1.),
        (1., 1., 1.),
        texture: texture.spiral(),
        n: 32,
        step: 0.01,
      ),
      func(
        (x, y) => 0.,
        (-1., -1., -1.),
        (1., 1., 1.),
        texture: texture.grid(grid-size: 0.2),
        n: 16,
        step: 0.01,
      ),
      sphere((0., -0.6, 0.), 0.25, texture: texture.random_circles(42)),
    ),
    width: 100%,
  )
}

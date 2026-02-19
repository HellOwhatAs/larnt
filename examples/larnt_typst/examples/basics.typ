#set page(margin: 0pt, height: auto)
#import "@preview/larnt:0.1.0": *

#image(
  render(
    eye: (2., 7., 5.),
    center: (1.5, 2., 0.),
    cube((0., 0., 0.), (1., 1., 1.)),
    cube((1.5, 0., 0.), (2.5, 1., 1.), texture: texture.striped(8)),
    sphere((0.5, 2., .5), 0.5),
    sphere((2., 2., .5), 0.5, texture: texture.random_circles(42)),
    sphere((0.5, 3.5, .5), 0.5, texture: texture.random_equators(42)),
    sphere((2., 3.5, .5), 0.5, texture: texture.lat_lng()),
    sphere((3.5, 3.5, .5), 0.5, texture: texture.random_fuzz(42)),
    cone(.5, (-1., .5, 0.), (-1., .5, 1.)),
    cone(.5, (-1., 2., 0.), (-1., 2., 1.), texture: texture.striped(15)),
    cylinder(.5, (3.5, .5, 0.), (3.5, .5, 1.)),
    cylinder(.5, (3.5, 2., 0.), (3.5, 2., 1.), texture: texture.striped(32)),
  ),
  width: 100%,
)

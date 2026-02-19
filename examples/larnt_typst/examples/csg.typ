#set page(margin: 0pt, height: auto)
#import "@preview/larnt:0.1.0": *

#image(
  render(
    eye: (3., 2.5, 2.0),
    center: (1., 0.0, 0.0),
    difference(
      cube((0., 0., 0.), (1., 1., 1.), texture: texture.striped(15)),
      sphere((1., 1., 0.5), 0.5, texture: texture.lat_lng()),
      sphere((0., 1., 0.5), 0.5, texture: texture.lat_lng()),
      sphere((0., 0., 0.5), 0.5, texture: texture.lat_lng()),
      sphere((1., 0., 0.5), 0.5, texture: texture.lat_lng()),
    ),
    cube((0.3, 0.3, 1.), (.7, .7, 1.2), texture: texture.striped(10)),
    intersection(
      sphere((2.0, 0.25, 0.5), 0.6, texture: texture.lat_lng()),
      cube((1.5, -0.25, 0.), (2.5, 0.75, 1.0), texture: texture.striped(25)),
    ),
  ),
  width: 100%,
)

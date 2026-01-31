#set page(margin: 0pt, height: auto)
#let ln = plugin("./ln_typst_plugin.wasm")
#import "@preview/suiji:0.5.1"

#let rng = suiji.gen-rng-f(42)
#let blocks = {
  let n = 10
  let res = ()
  for x in range(-n, n + 1) {
    for y in range(-n, n + 1) {
      let p
      let fz
      (rng, (p, fz)) = suiji.random-f(rng, size: 2)
      p = p * 0.25 + 0.2
      fz = fz * 3.0 + 1.0
      if x == 2 and y == 1 {
        continue
      }
      res.push(
        ((x - p, y - p, 0.0), (x + p, y + p, fz)),
      )
    }
  }
  res
}

#image(ln.render(
  cbor.encode((
    eye: (1.75, 1.25, 6.0),
    center: (0.0, 0.0, 0.0),
    up: (0.0, 0.0, 1.0),
    width: 1024.0,
    height: 1024.0,
    fovy: 100.0,
    near: 0.1,
    far: 30.0,
    step: 0.01,
  )),
  cbor.encode(blocks),
))

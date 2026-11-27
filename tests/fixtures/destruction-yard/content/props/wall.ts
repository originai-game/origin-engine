type model
version 1
---
# A brick wall, 5 x 2.4 x 0.3 m, anchored at its foot and both ends: a
# running bond of 0.25 x 0.1 m bricks that breaks along its mortar into runs
# of one to three bricks, each falling apart into its bricks. Plain colours:
# the fixture's frames read the break, not a texture.
material "brick" {
  color 0.58 0.3 0.22
  roughness 0.9
}
part "wall" {
  material "brick"
  position 0 1.2 0
  boxGeometry {
    width 5
    height 2.4
    depth 0.3
  }
}
fracture {
  style brick
  brick 0.25 0.1
  chunks 250
  seed 11
  children 32
  anchor bottom ends
  interior {
    color 0.66 0.38 0.28
    roughness 1
  }
  mortar {
    color 0.66 0.63 0.57
    roughness 1
  }
}

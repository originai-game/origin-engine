type model
version 1
---
# The level's and the cutscene's set, from built-in geometry.
material "floor" {
  color 0.3 0.28 0.25
  roughness 0.9
}
material "stone" {
  color 0.55 0.5 0.42
  roughness 0.7
}
part "floor" {
  material "floor"
  position 0 -0.05 0
  boxGeometry {
    width 12
    height 0.1
    depth 12
  }
}
part "pillar" {
  material "stone"
  position -1.5 1.2 -1
  boxGeometry {
    width 0.8
    height 2.4
    depth 0.8
  }
}

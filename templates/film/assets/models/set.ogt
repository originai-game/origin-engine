type model
version 1
---
# The live ending's set, built from built-in geometry (og api show model):
# replace it with the game's own models; the segment keeps its camera move.
material "floor" {
  color 0.08 0.07 0.06
  roughness 0.9
}
material "stone" {
  color 0.35 0.31 0.26
  roughness 0.7
}
material "lamp" {
  color 1 0.78 0.45
  emissive 1 0.7 0.35
  emissiveIntensity 1.5
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
  position -1.2 1.2 0
  boxGeometry {
    width 0.8
    height 2.4
    depth 0.8
  }
}
part "plinth" {
  material "stone"
  position 1 0.35 0.4
  cylinderGeometry {
    radiusTop 0.6
    radiusBottom 0.7
    height 0.7
  }
}
part "lamp" {
  material "lamp"
  position 1 0.95 0.4
  sphereGeometry {
    radius 0.25
  }
}

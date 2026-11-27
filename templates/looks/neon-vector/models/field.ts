type model
version 1
---
# The shape family's ground: the art brief of its styles asks for untextured
# flat colour (faceted, clay, paper, minimal), so the field is one flat
# material, not the textured glb the other families take.
material "grass" {
  color 0.02 0.02 0.04
  roughness 1
}
part "field" {
  material "grass"
  quaternion -0.7071068 0 0 0.7071068
  planeGeometry {
    width 40
    height 40
  }
}

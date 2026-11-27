type model
version 1
---
material "ball" {
  color 0.8 0.2 0.15
  roughness 0.4
}
part "ball" {
  material "ball"
  sphereGeometry {
    radius 0.3
  }
}

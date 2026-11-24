type model
version 1
---
material "iron" {
  color 0.03 0.035 0.04
  roughness 0.5
  metalness 0.6
}
material "light" {
  color 1 0.85 0.6
  emissive 1 0.8 0.5
  emissiveIntensity 2
}
part "pole" {
  material "iron"
  cylinderGeometry {
    radiusTop 0.06
    radiusBottom 0.09
    height 4.2
    radialSegments 8
  }
  at {
    position 0 2.1 0
  }
}
part "head" {
  material "light"
  sphereGeometry {
    radius 0.22
    widthSegments 10
    heightSegments 6
  }
  at {
    position 0 4.3 0
  }
}

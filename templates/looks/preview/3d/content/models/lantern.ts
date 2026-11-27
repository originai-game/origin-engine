type model
version 1
---
# The street lantern: an iron post 2.6 m tall, an arm and a lit glass head.
material "iron" {
  color 0.1 0.1 0.11
  roughness 0.45
  metalness 0.7
}
material "glass" {
  color 1 0.75 0.35
  emissive 1 0.6 0.2
  emissiveIntensity 5
}
part "base" {
  material "iron"
  position 0 0.12 0
  boxGeometry {
    width 0.32
    height 0.24
    depth 0.32
  }
}
part "post" {
  material "iron"
  position 0 1.35 0
  cylinderGeometry {
    radiusTop 0.05
    radiusBottom 0.07
    height 2.3
    radialSegments 12
  }
}
part "arm" {
  material "iron"
  position 0.22 2.45 0
  boxGeometry {
    width 0.5
    height 0.06
    depth 0.06
  }
}
part "frame" {
  material "iron"
  position 0.42 2.15 0
  boxGeometry {
    width 0.3
    height 0.06
    depth 0.3
  }
}
part "glass" {
  material "glass"
  position 0.42 2.3 0
  boxGeometry {
    width 0.24
    height 0.26
    depth 0.24
  }
}
part "cap" {
  material "iron"
  position 0.42 2.5 0
  coneGeometry {
    radius 0.22
    height 0.18
    radialSegments 4
  }
}

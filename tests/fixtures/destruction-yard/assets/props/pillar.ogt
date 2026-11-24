type model
version 1
---
# A concrete pillar, 0.5 m across and 3 m tall, anchored at its foot: a blow
# at its middle takes that band away in lumps and the top falls onto the
# stump.
material "concrete" {
  color 0.62 0.61 0.58
  roughness 0.85
}
part "shaft" {
  material "concrete"
  position 0 1.5 0
  cylinderGeometry {
    radiusTop 0.25
    radiusBottom 0.25
    height 3
    radialSegments 20
  }
}
fracture {
  style concrete
  chunks 44
  seed 5
  children 5
  interior {
    color 0.5 0.49 0.46
    roughness 0.95
  }
}

type model
version 1
---
# The stepped tree: a trunk under three tiers, each smaller than the one
# below (og-model §5), about 3.4 m tall, its foot on y = 0.
material "bark" {
  color 0.33 0.2 0.1
  roughness 0.9
}
material "leaves" {
  color 0.15 0.42 0.2
  roughness 0.85
}
part "trunk" {
  material "bark"
  position 0 0.6 0
  cylinderGeometry {
    radiusTop 0.12
    radiusBottom 0.18
    height 1.2
    radialSegments 12
  }
}
part "tier-0" {
  material "leaves"
  position 0 1.55 0
  coneGeometry {
    radius 1.15
    height 1.1
    radialSegments 18
  }
}
part "tier-1" {
  material "leaves"
  position 0 2.25 0
  coneGeometry {
    radius 0.85
    height 0.95
    radialSegments 18
  }
}
part "tier-2" {
  material "leaves"
  position 0 2.85 0
  coneGeometry {
    radius 0.55
    height 0.85
    radialSegments 18
  }
}

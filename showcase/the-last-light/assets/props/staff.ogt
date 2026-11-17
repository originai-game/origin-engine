type model
version 1
---
# The wanderer's staff: a weathered ash pole with a crook at its head, a
# cloth grip where the hand holds it and an iron hook for the lantern.
# Its foot is the origin; the lantern hangs from the hook at 0.13 1.74 0.
material "ash" {
  color 0.2 0.13 0.075
  roughness 0.82
}
material "wrap" {
  color 0.46 0.1 0.07
  roughness 0.95
}
material "iron" {
  color 0.05 0.05 0.055
  roughness 0.5
  metalness 1
}
part "shaft" {
  material "ash"
  position 0 0.95 0
  cylinderGeometry {
    radiusTop 0.017
    radiusBottom 0.024
    height 1.9
    radialSegments 9
    heightSegments 6
  }
}
part "crook" {
  material "ash"
  position 0.075 1.9 0
  quaternion 0 0 -0.2474 0.9689
  torusGeometry {
    radius 0.075
    tube 0.016
    radialSegments 8
    tubularSegments 20
    arc 3.64
  }
}
part "grip" {
  material "wrap"
  position 0 1.32 0
  cylinderGeometry {
    radiusTop 0.025
    radiusBottom 0.027
    height 0.2
    radialSegments 9
  }
}
part "hook" {
  material "iron"
  position 0.06 1.8 0
  quaternion 0 0 0.7071 0.7071
  cylinderGeometry {
    radiusTop 0.005
    radiusBottom 0.005
    height 0.14
    radialSegments 6
  }
}
part "ferrule" {
  material "iron"
  position 0 0.04 0
  cylinderGeometry {
    radiusTop 0.026
    radiusBottom 0.02
    height 0.08
    radialSegments 9
  }
}

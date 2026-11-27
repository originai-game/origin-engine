type model
version 1
---
# A broadleaf tree, about 3.6 m: a turned trunk under three round crowns of
# smooth spheres, so a look's ArtShape shows on curved surfaces (facets,
# a bevel, voxels or clay) as well as on the stepped pine's cones.
material "bark" {
  color 0.3 0.19 0.1
  roughness 0.9
}
material "leaves" {
  color 0.22 0.45 0.16
  roughness 0.8
}
part "trunk" {
  material "bark"
  position 0 0.9 0
  cylinderGeometry {
    radiusTop 0.13
    radiusBottom 0.22
    height 1.8
    radialSegments 14
  }
}
part "crown" {
  material "leaves"
  sphereGeometry {
    radius 1
    widthSegments 24
    heightSegments 16
  }
  at {
    position 0 2.55 0
    scale 1.15 1 1.15
  }
  at {
    position -0.65 2.15 0.3
    scale 0.75 0.7 0.75
  }
  at {
    position 0.6 2.25 -0.25
    scale 0.8 0.72 0.8
  }
}

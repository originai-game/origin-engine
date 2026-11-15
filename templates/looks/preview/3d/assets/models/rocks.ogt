type model
version 1
---
# Two boulders and a bush beside the path: smooth, round forms whose shading
# runs from lit to shade, where a look's facets, bands or material show.
material "stone" {
  color 0.46 0.44 0.4
  roughness 0.85
}
material "bush" {
  color 0.2 0.4 0.15
  roughness 0.85
}
part "boulder" {
  material "stone"
  icosahedronGeometry {
    radius 0.5
    detail 3
  }
  at {
    position 0 0.32 0
    scale 1.3 0.85 1
  }
  at {
    position 0.85 0.2 0.35
    scale 0.7 0.55 0.65
  }
}
part "bush" {
  material "bush"
  sphereGeometry {
    radius 0.5
    widthSegments 20
    heightSegments 14
  }
  at {
    position -1.1 0.3 -0.2
    scale 1.2 0.85 1.1
  }
  at {
    position -0.55 0.25 0.45
    scale 0.8 0.65 0.8
  }
}

type model
version 1
---
# The far layer: a ridge of rounded hills and a small village, 20 to 30 m
# behind the scene, in muted colours so the beats in front read. As the
# low-resolution-3d brief asks for a prop (ArtShape.propFaces 300): each
# hill a coarse 8 x 4 sphere, the faceted ridge of early-console scenery.
material "hill" {
  color 0.24 0.36 0.3
  roughness 1
}
material "far-hill" {
  color 0.3 0.38 0.45
  roughness 1
}
material "wall" {
  color 0.75 0.7 0.6
  roughness 0.9
}
material "roof" {
  color 0.55 0.22 0.15
  roughness 0.9
}
part "hill-0" {
  material "far-hill"
  position -14 -2 -8
  scale 12 6 4
  sphereGeometry {
    radius 1
    widthSegments 6
    heightSegments 4
  }
}
part "hill-1" {
  material "far-hill"
  position 10 -2.5 -9
  scale 15 7.5 4
  sphereGeometry {
    radius 1
    widthSegments 6
    heightSegments 4
  }
}
part "hill-2" {
  material "hill"
  position -3 -3 -2
  scale 13 5 4
  sphereGeometry {
    radius 1
    widthSegments 6
    heightSegments 4
  }
}
part "hill-3" {
  material "hill"
  position 16 -3 -1
  scale 9 4.5 4
  sphereGeometry {
    radius 1
    widthSegments 6
    heightSegments 4
  }
}
part "house" {
  material "wall"
  boxGeometry {
    width 1.6
    height 1.2
    depth 1.2
  }
  at {
    position 4 1.6 0
  }
  at {
    position 6.2 1.4 0.6
  }
  at {
    position 2 1.3 1
  }
}
part "roof" {
  material "roof"
  coneGeometry {
    radius 1.2
    height 0.9
    radialSegments 4
  }
  at {
    position 4 2.65 0
    quaternion 0 0.3827 0 0.9239
  }
  at {
    position 6.2 2.45 0.6
    quaternion 0 0.3827 0 0.9239
  }
  at {
    position 2 2.35 1
    quaternion 0 0.3827 0 0.9239
  }
}
part "tower" {
  material "wall"
  position 5 2.4 -0.6
  boxGeometry {
    width 0.7
    height 3
    depth 0.7
  }
}

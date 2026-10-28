type model
version 1
---
# neon-vector's version (tools/art-samples/neon.py): black faces, glowing hues.
# The far layer: a ridge of rounded hills and a small village, 20 to 30 m
# behind the scene, in muted colours so the beats in front read.
material "hill" {
  color 0.02 0.02 0.04
  roughness 1
  emissive 0.4 0.3 1
  emissiveIntensity 0.6
}
material "far-hill" {
  color 0.02 0.02 0.04
  roughness 1
  emissive 0.3 0.2 0.8
  emissiveIntensity 0.6
}
material "wall" {
  color 0.02 0.02 0.04
  roughness 1
  emissive 0.1 0.9 1
  emissiveIntensity 0.6
}
material "roof" {
  color 0.02 0.02 0.04
  roughness 1
  emissive 1 0.3 0.8
  emissiveIntensity 0.6
}
part "hill-0" {
  material "far-hill"
  position -14 -2 -8
  scale 12 6 4
  sphereGeometry {
    radius 1
    widthSegments 24
    heightSegments 12
  }
}
part "hill-1" {
  material "far-hill"
  position 10 -2.5 -9
  scale 15 7.5 4
  sphereGeometry {
    radius 1
    widthSegments 24
    heightSegments 12
  }
}
part "hill-2" {
  material "hill"
  position -3 -3 -2
  scale 13 5 4
  sphereGeometry {
    radius 1
    widthSegments 24
    heightSegments 12
  }
}
part "hill-3" {
  material "hill"
  position 16 -3 -1
  scale 9 4.5 4
  sphereGeometry {
    radius 1
    widthSegments 24
    heightSegments 12
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

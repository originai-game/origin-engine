type effect
version 1
---
# The heal's column: noise climbing the built-in cylinder (whose open ends
# fade), soft at its silhouette and eroded as it fades.
main "textures/noise.png" red {
  tile 2 1
  scroll 0 -0.8
}
ramp {
  key 0 0.02 0.15 0.05 0.2
  key 0.3 0.15 1.2 0.4 0.6
  key 0.65 0.6 3 1.2 1
  key 1 1.5 5 2.5 1
}
noise "textures/noise.png" red {
  tile 1 2
  scroll 0 -0.5
}
erosion curve 0 0.6 0.15 0.1 0.8 0.15 1 1
erosionSoftness 0.3
fresnel 3 1 invert
intensity 4
blend additive
doubleSided true

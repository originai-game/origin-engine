type effect
version 1
---
# The hit's shock ring: noise streaming outward in polar coordinates,
# eroded as the ring grows.
main "textures/noise.png" red {
  tile 5 0.6
  scroll 0.1 -1.2
  polar true
}
ramp {
  key 0 0 0 0 0
  key 0.4 1.2 0.35 0.05 0.6
  key 0.8 4 1.8 0.5 1
  key 1 6 4 2.5 1
}
noise "textures/noise.png" red {
  tile 2 2
}
erosion curve 0 0.05 1 0.9
erosionSoftness 0.25
intensity 1.4
blend additive
doubleSided true

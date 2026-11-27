type vfx
version 2
---
duration 30
decal {
  until 30
  fade 5
  texture "vfx/cinematic-realism/textures/scorch.png"
  normal "vfx/cinematic-realism/textures/scorch-normal.png"
  size 1.6 0.6 1.6
  roughness 0.96
  angleFade 80
  turn 0 360
}
decal {
  life 5
  fade 4.5
  texture "vfx/cinematic-realism/textures/embers.png"
  size 0.8 0.6 0.8
  color 0.06 0.05 0.045 1
  emissive 3 1.7 0.9
  angleFade 80
  turn 0 360
}

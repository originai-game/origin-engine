type vfx
version 2
---
duration 1.5
loop true
emitter "vfx/cinematic-realism/emitters/drip-drop.ogt" {
  offset 0 2.2 0
}
mesh ring {
  at 0.67
  life 0.8
  offset 0 0.005 0
  rotate -90 0 0
  segments 32
  inner 0.7
  colors 1 1 1 0 1 1 1 0.85 1 1 1 0
  effect "vfx/cinematic-realism/water.ogt"
  scale curve 0 0.02 0.4 0.1 1 0.16
  color 0.8 0.88 0.95
  blend alpha
}
decal {
  life 1.5
  texture "vfx/cinematic-realism/textures/wet.png"
  wet true
  size 0.32 0.2 0.32
  color 0.32 0.33 0.35 1
  roughness 0.12
  turn 0 360
}

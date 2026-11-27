type vfx
version 2
---
duration 18
impact 0
param mark 1
emitter "vfx/cinematic-realism/emitters/fireball-large.ogt" {
  offset 0 3 0
}
emitter "vfx/cinematic-realism/emitters/smoke-column-large.ogt" {
  offset 0 4 0
}
emitter "vfx/cinematic-realism/emitters/dust-ring-large.ogt" {
  offset 0 0.3 0
}
emitter "vfx/cinematic-realism/emitters/debris-large.ogt" {
  offset 0 0.4 0
}
emitter "vfx/cinematic-realism/emitters/sparks.ogt" {
  offset 0 1 0
}
emitter "vfx/cinematic-realism/emitters/heat-large.ogt" {
  offset 0 3.5 0
}
glow {
  life 0.25
  offset 0 2.5 0
  size curve 0 4 0.3 9 1 6
  color 2.2 1.1 0.4 0.7
}
light {
  life 1
  offset 0 3 0
  color 1 0.5 0.18
  intensity curve 0 30 0.15 18 1 0
  distance 25
}
shake {
  trauma 0.8
}
decal {
  at 0.04
  until 18
  fade 5
  when mark
  texture "vfx/cinematic-realism/textures/scorch.png"
  normal "vfx/cinematic-realism/textures/scorch-normal.png"
  size 18 3 18
  roughness 0.96
  angleFade 80
  turn 0 360
}
decal {
  at 0.12
  life 8
  fade 7
  when mark
  texture "vfx/cinematic-realism/textures/embers.png"
  size 8 3 8
  color 0.06 0.05 0.045 1
  emissive 3 1.7 0.9
  angleFade 80
  turn 0 360
}
sound "base/vfx/cinematic-realism/audio/blast-large" {
  playbackRate 0.95 1.05
}

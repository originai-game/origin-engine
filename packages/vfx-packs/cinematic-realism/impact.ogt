type vfx
version 2
---
duration 10
impact 0
param mark 1
emitter "vfx/cinematic-realism/emitters/impact-sparks.ogt" {
}
emitter "vfx/cinematic-realism/emitters/impact-dust.ogt" {
  offset 0 0.05 0
}
emitter "vfx/cinematic-realism/emitters/grit.ogt" {
  offset 0 0.03 0
}
glow {
  life 0.16
  offset 0 0.12 0
  size curve 0 0.18 0.2 0.36 1 0
  color 1.6 0.85 0.25 0.65
}
light {
  life 0.12
  offset 0 0.4 0
  color 1 0.62 0.28
  intensity curve 0 0.18 1 0
  distance 1.8
}
shake {
  trauma 0.18
}
decal {
  until 10
  fade 3
  when mark
  texture "vfx/cinematic-realism/textures/dent.png"
  normal "vfx/cinematic-realism/textures/dent-normal.png"
  normalScale 1.5
  size 0.14 0.1 0.14
  roughness 0.5
  turn 0 360
}
sound "base/vfx/cinematic-realism/audio/tick" {
  volume 0.9
  playbackRate 1 1.25
}

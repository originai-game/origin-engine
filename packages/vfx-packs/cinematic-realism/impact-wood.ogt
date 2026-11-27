type vfx
version 2
---
duration 10
impact 0
param mark 1
emitter "vfx/cinematic-realism/emitters/splinters.ogt" {
  offset 0 0.03 0
}
emitter "vfx/cinematic-realism/emitters/wood-dust.ogt" {
  offset 0 0.04 0
}
decal {
  until 10
  fade 3
  when mark
  texture "vfx/cinematic-realism/textures/gouge.png"
  normal "vfx/cinematic-realism/textures/gouge-normal.png"
  normalScale 2
  size 0.3 0.2 0.3
  roughness 0.85
  turn 0 360
}
sound "base/vfx/cinematic-realism/audio/crack" {
  volume 0.6
  playbackRate 1.1 1.4
}

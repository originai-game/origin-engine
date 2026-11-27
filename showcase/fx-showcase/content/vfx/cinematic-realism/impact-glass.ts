type vfx
version 2
---
duration 10
impact 0
param mark 1
emitter "vfx/cinematic-realism/emitters/glass-shards.ogt" {
  offset 0 0.03 0
}
emitter "vfx/cinematic-realism/emitters/glints.ogt" {
  offset 0 0.03 0
}
decal {
  until 10
  fade 3
  when mark
  texture "vfx/cinematic-realism/textures/fracture.png"
  normal "vfx/cinematic-realism/textures/fracture-normal.png"
  size 0.5 0.2 0.5
  color 1 1 1 0.9
  roughness 0.25
  turn 0 360
}
sound "base/vfx/cinematic-realism/audio/shatter" {
  volume 0.5
  playbackRate 1 1.2
}

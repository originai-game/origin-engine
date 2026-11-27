type vfx
version 2
---
duration 1.2
impact 0
emitter "vfx/cinematic-realism/emitters/plip-jet.ogt" {
  offset 0 0.02 0
}
emitter "vfx/cinematic-realism/emitters/plip-mist.ogt" {
  offset 0 0.05 0
}
mesh ring {
  life 0.9
  offset 0 0.006 0
  rotate -90 0 0
  segments 32
  inner 0.84
  colors 1 1 1 0 1 1 1 0.5 1 1 1 0
  effect "vfx/cinematic-realism/water.ogt"
  scale curve 0 0.03 0.4 0.14 1 0.24
  color 0.8 0.88 0.95
  blend alpha
}
sound "base/vfx/cinematic-realism/audio/plip" {
  playbackRate 0.9 1.2
}

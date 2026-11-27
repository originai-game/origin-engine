type vfx
version 2
---
duration 1.2
impact 0
mesh ring {
  life 0.35
  offset 0 0.005 0
  rotate -90 0 0
  segments 32
  inner 0.8
  height 0.3 0
  effect "vfx/cinematic-realism/crown.ogt"
  scale curve 0 0.04 0.4 0.13 1 0.17
}
emitter "vfx/cinematic-realism/emitters/plip-jet.ogt" {
  offset 0 0.02 0
}
emitter "vfx/cinematic-realism/emitters/spray-small.ogt" {
  offset 0 0.03 0
}
emitter "vfx/cinematic-realism/emitters/plip-mist.ogt" {
  offset 0 0.05 0
}
mesh ring {
  life 1
  offset 0 0.006 0
  rotate -90 0 0
  segments 48
  inner 0.86
  colors 1 1 1 0 1 1 1 0.5 1 1 1 0
  effect "vfx/cinematic-realism/water.ogt"
  scale curve 0 0.06 0.3 0.2 1 0.36
  color 0.8 0.88 0.95
  blend alpha
}
mesh ring {
  at 0.2
  life 1
  offset 0 0.005 0
  rotate -90 0 0
  segments 48
  inner 0.88
  colors 1 1 1 0 1 1 1 0.4 1 1 1 0
  effect "vfx/cinematic-realism/water.ogt"
  scale curve 0 0.04 0.4 0.14 1 0.24
  color 0.8 0.88 0.95
  blend alpha
}
sound "base/vfx/cinematic-realism/audio/splash" {
  volume 0.6
  playbackRate 1.4 1.7
}

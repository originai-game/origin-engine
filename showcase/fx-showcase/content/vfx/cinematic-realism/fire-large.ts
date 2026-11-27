type vfx
version 2
---
duration 2
loop true
emitter "vfx/cinematic-realism/emitters/flame-large.ogt" {
  offset 0 0.9 0
}
emitter "vfx/cinematic-realism/emitters/flame-small.ogt" {
  offset 0.6 0.25 0.2
}
emitter "vfx/cinematic-realism/emitters/flame-small.ogt" {
  offset -0.5 0.25 -0.3
  seed 5
}
emitter "vfx/cinematic-realism/emitters/fire-smoke-heavy.ogt" {
  offset 0 2 0
}
emitter "vfx/cinematic-realism/emitters/fire-embers.ogt" {
  offset 0 1 0
}
emitter "vfx/cinematic-realism/emitters/fire-embers.ogt" {
  offset 0.3 1.4 0.2
  seed 9
}
emitter "vfx/cinematic-realism/emitters/heat-rise.ogt" {
  offset 0 1.8 0
}
glow {
  life 2
  offset 0 1 0
  size 2.6
  color 1.6 0.8 0.25 0.12
}
light {
  life 2
  offset 0 1.4 0
  color 1 0.55 0.22
  intensity curve 0 6 0.2 7.5 0.45 5.5 0.7 7 1 6
  distance 14
}

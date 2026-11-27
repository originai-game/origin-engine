type vfx
version 2
---
duration 2
loop true
emitter "vfx/cinematic-realism/emitters/flame-small.ogt" {
  offset 0 0.25 0
}
emitter "vfx/cinematic-realism/emitters/flame-small.ogt" {
  offset 0.08 0.2 0.06
  seed 5
}
emitter "vfx/cinematic-realism/emitters/fire-embers.ogt" {
  offset 0 0.3 0
}
emitter "vfx/cinematic-realism/emitters/fire-smoke.ogt" {
  offset 0 0.7 0
}
emitter "vfx/cinematic-realism/emitters/heat-rise.ogt" {
  offset 0 0.6 0
}
glow {
  life 2
  offset 0 0.3 0
  size 0.7
  color 1.6 0.8 0.25 0.18
}
light {
  life 2
  offset 0 0.45 0
  color 1 0.55 0.22
  intensity curve 0 1.8 0.2 2.3 0.45 1.6 0.7 2.1 1 1.8
  distance 6
}

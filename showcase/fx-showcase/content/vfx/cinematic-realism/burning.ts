type vfx
version 2
---
duration 2
loop true
emitter "vfx/cinematic-realism/emitters/flame-body.ogt" {
  offset 0 0.8 0
}
emitter "vfx/cinematic-realism/emitters/fire-embers.ogt" {
  offset 0 1.2 0
}
emitter "vfx/cinematic-realism/emitters/wisp-up.ogt" {
  offset 0 1.5 0
}
light {
  life 2
  offset 0 1 0
  color 1 0.55 0.22
  intensity curve 0 1.6 0.2 2.1 0.45 1.4 0.7 1.9 1 1.6
  distance 5
}

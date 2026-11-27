type vfx
version 2
---
duration 2
loop true
emitter "vfx/cinematic-realism/emitters/flame-jet.ogt" {
}
emitter "vfx/cinematic-realism/emitters/flame-soot.ogt" {
}
emitter "vfx/cinematic-realism/emitters/heat-jet.ogt" {
}
glow {
  life 2
  offset 0 0.1 0
  size 0.3
  color 2.4 1.4 0.5 0.6
}
light {
  life 2
  offset 0 1.8 0
  color 1 0.55 0.2
  intensity curve 0 2 0.3 2.6 0.6 1.8 1 2
  distance 7
}

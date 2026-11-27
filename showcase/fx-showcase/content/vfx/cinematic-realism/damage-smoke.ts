type vfx
version 2
---
duration 2
loop true
param fire 0
emitter "vfx/cinematic-realism/emitters/damage-smoke.ogt" {
  offset 0 0.2 0
}
emitter "vfx/cinematic-realism/emitters/flame-small.ogt" {
  offset 0 0.25 0
  when fire
}
emitter "vfx/cinematic-realism/emitters/spark-spits.ogt" {
  when fire
}
light {
  life 2
  offset 0 0.4 0
  color 1 0.55 0.2
  intensity curve 0 1 0.3 1.4 0.6 0.9 1 1
  distance 4
  when fire
}

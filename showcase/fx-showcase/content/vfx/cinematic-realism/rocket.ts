type vfx
version 2
---
duration 2
loop true
emitter "vfx/cinematic-realism/emitters/rocket-flame.ogt" {
  rotate 180 0 0
}
emitter "vfx/cinematic-realism/emitters/rocket-smoke.ogt" {
  offset 0 -0.25 0
}
emitter "vfx/cinematic-realism/emitters/rocket-sparks.ogt" {
  offset 0 -0.1 0
}
glow {
  life 2
  offset 0 -0.1 0
  size 0.4
  color 2.4 1.3 0.5 0.7
}
light {
  life 2
  offset 0 -0.3 0
  color 1 0.6 0.25
  intensity 1.2
  distance 5
}

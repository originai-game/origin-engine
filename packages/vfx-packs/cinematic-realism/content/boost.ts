type vfx
version 2
---
duration 1.2
impact 0
emitter "vfx/cinematic-realism/emitters/boost-flame.ogt" {
}
emitter "vfx/cinematic-realism/emitters/heat-jet.ogt" {
  offset 0 0.5 0
  until 0.5
}
glow {
  life 0.5
  offset 0 0.2 0
  size curve 0 0.7 0.2 0.45 1 0.15
  color 0.7 1 2 0.5
}
light {
  life 0.5
  offset 0 0.5 0
  color 0.6 0.8 1
  intensity curve 0 3 1 0
  distance 6
}
shake {
  trauma 0.2
}
sound "base/vfx/cinematic-realism/audio/whoosh" {
}

type vfx
version 2
---
duration 1.5
loop true
emitter "vfx/cinematic-realism/emitters/inward-motes.ogt" {
}
glow {
  life 1.5
  size curve 0 0.2 0.5 0.3 1 0.2
  color 0.8 1.5 3.2 0.8
}
glow {
  life 1.5
  size curve 0 0.6 0.5 0.8 1 0.6
  color 0.2 0.5 1.4 0.3
}
light {
  life 1.5
  color 0.45 0.7 1
  intensity curve 0 0.8 0.5 1.3 1 0.8
  distance 4
}

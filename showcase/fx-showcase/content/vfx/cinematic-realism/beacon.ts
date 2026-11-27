type vfx
version 2
---
duration 2
loop true
beam {
  life 2
  to 0 6 0
  width 0.5
  effect "vfx/cinematic-realism/beam-soft.ogt"
  color 1.2 0.95 0.5 0.5
}
beam {
  life 2
  to 0 9 0
  width 0.14
  effect "vfx/cinematic-realism/beam-soft.ogt"
  color 2 1.6 0.9 0.6
}
emitter "vfx/cinematic-realism/emitters/beacon-motes.ogt" {
  offset 0 0.1 0
}
glow {
  life 2
  offset 0 0.1 0
  size curve 0 0.7 0.5 0.85 1 0.7
  color 1.6 1.2 0.5 0.4
}
light {
  life 2
  offset 0 0.5 0
  color 1 0.8 0.45
  intensity 0.8
  distance 4
}

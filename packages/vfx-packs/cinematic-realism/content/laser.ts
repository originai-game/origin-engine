type vfx
version 2
---
duration 1
loop true
param length 10
beam {
  life 1
  to 0 1 0
  stretch length
  width 0.035
  effect "vfx/cinematic-realism/beam.ogt"
  color 3.2 0.5 0.4 1
}
beam {
  life 1
  to 0 1 0
  stretch length
  width 0.14
  effect "vfx/cinematic-realism/beam.ogt"
  color 0.9 0.1 0.08 0.35
}
glow {
  life 1
  size 0.14
  color 3 0.7 0.5 0.7
}
light {
  life 1
  color 1 0.3 0.25
  intensity 0.4
  distance 2
}

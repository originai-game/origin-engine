type vfx
version 2
---
duration 1
loop true
param length 1
beam {
  life 1
  to 0 -3 0
  stretch length
  width 0.03
  effect "vfx/cinematic-realism/tracer-body.ogt"
  color 3 1.3 0.45 1
}
beam {
  life 1
  to 0 -1.2 0
  stretch length
  width 0.12
  effect "vfx/cinematic-realism/tracer-body.ogt"
  color 1.2 0.4 0.12 0.35
}
glow {
  life 1
  size 0.06
  color 3 1.6 0.7 0.8
}

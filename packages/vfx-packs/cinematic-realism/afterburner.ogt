type vfx
version 2
---
duration 2
loop true
param thrust 1
emitter "vfx/cinematic-realism/emitters/burner-core.ogt" {
  amount thrust
}
emitter "vfx/cinematic-realism/emitters/burner-wake.ogt" {
  amount thrust
}
emitter "vfx/cinematic-realism/emitters/heat-jet.ogt" {
  offset 0 0.6 0
}
glow {
  life 2
  offset 0 0.2 0
  size 0.3
  stretch thrust
  color 0.6 0.9 1.8 0.4
}
glow {
  life 2
  offset 0 0.7 0
  size 0.22
  amount thrust
  color 1.5 0.95 0.4 0.4
}
glow {
  life 2
  offset 0 1.2 0
  size 0.18
  amount thrust
  color 1.4 0.7 0.22 0.35
}
glow {
  life 2
  offset 0 1.7 0
  size 0.13
  amount thrust
  color 1.1 0.45 0.13 0.3
}
light {
  life 2
  offset 0 0.6 0
  color 1 0.65 0.35
  intensity 2
  amount thrust
  distance 6
}

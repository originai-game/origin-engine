type vfx
version 2
---
# Converted from Effekseer by og vfx import.
duration 2.05
impact 0
emitter "vfx/effekseer-starter/MagicDark/n0.emitter.ogt" {
  at 0
  offset 0 1.5 0
}
mesh quad {
  at 0
  rotate -90 0 0
  life 1.6667
  effect "vfx/effekseer-starter/MagicDark/n1-0.effect.ogt"
  scale 3
  spin 0 0 120
  color 0.586 0.145 1 1
  blend additive
}
emitter "vfx/effekseer-starter/MagicDark/n2.emitter.ogt" {
  at 0.0333
  offset 0 1 0
}
light {
  at 0
  life 0.3
  offset 0 0.1111 0
  color 0.6166 0.2083 1
  intensity curve 0 8 1 0
  distance 12
}
shake {
  at 0
  trauma 0.4
}

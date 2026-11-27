type vfx
version 2
---
duration 0.5
impact 0
emitter "vfx/geometric/emitters/chips.ogt" {
}
mesh ring {
  life 0.22
  rotate -90 0 0
  offset 0 0.06 0
  segments 5
  inner 0.78
  effect "vfx/geometric/flat.ogt"
  scale curve 0 0.08 0.3 0.35 1 0.5
  color 0.9 0.22 0.12
  blend alpha
}
shake {
  trauma 0.16
}

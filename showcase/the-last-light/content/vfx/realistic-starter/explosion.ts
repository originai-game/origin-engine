type vfx
version 2
---
# A ground explosion, realistic: a fireball, a blast ring, fragments, debris, ground dust, rising smoke, a scorch, a flash of light and a shake. It stands on its origin: spawn it on the ground.
duration 4
impact 0
glow {
  at 0
  life 0.15
  offset 0 0.7 0
  size curve 0 1.4 1 2.4
  color 3 1.7 0.7
}
emitter "vfx/realistic-starter/explosion/fireball.emitter.ogt" {
  at 0
  offset 0 0.7 0
}
mesh ring {
  at 0
  life 0.35
  offset 0 0.05 0
  rotate -90 0 0
  inner 0.82
  effect "vfx/realistic-starter/explosion/shock.effect.ogt"
  scale curve 0 0.5 1 5.5
  color 1 1 1 1
  blend additive
}
emitter "vfx/realistic-starter/explosion/sparks.emitter.ogt" {
  at 0
  offset 0 0.5 0
}
emitter "vfx/realistic-starter/explosion/debris.emitter.ogt" {
  at 0
  offset 0 0.2 0
}
emitter "vfx/realistic-starter/explosion/dust.emitter.ogt" {
  at 0
  offset 0 0.15 0
}
emitter "vfx/realistic-starter/explosion/smoke.emitter.ogt" {
  at 0
  offset 0 0.9 0
}
emitter "vfx/realistic-starter/explosion/scorch.emitter.ogt" {
  at 0
  offset 0 0.01 0
}
light {
  at 0
  life 1.2
  offset 0 1.2 0
  color 1 0.55 0.22
  intensity curve 0 40 0.15 16 0.5 5 1 0
  distance 16
}
shake {
  at 0
  trauma 0.6
}

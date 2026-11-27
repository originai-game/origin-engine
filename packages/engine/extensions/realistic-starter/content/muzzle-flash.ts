type vfx
version 2
---
# A gunshot's muzzle flash, realistic: the flash, the burning plume, powder sparks and smoke. It fires along the effect's -z (the barrel's forward): spawn it at the muzzle turned with the gun.
duration 1.2
impact 0
emitter "vfx/realistic-starter/muzzle-flash/flash.emitter.ogt" {
  at 0
}
emitter "vfx/realistic-starter/muzzle-flash/plume.emitter.ogt" {
  at 0
  rotate -90 0 0
}
emitter "vfx/realistic-starter/muzzle-flash/sparks.emitter.ogt" {
  at 0
  rotate -90 0 0
}
emitter "vfx/realistic-starter/muzzle-flash/smoke.emitter.ogt" {
  at 0
  offset 0 0 -0.05
  rotate -90 0 0
}
glow {
  at 0
  life 0.05
  offset 0 0 -0.08
  size curve 0 0.22 1 0.3
  color 2.4 1.4 0.6
}
light {
  at 0
  life 0.07
  offset 0 0 -0.1
  color 1 0.62 0.3
  intensity curve 0 6 1 0
  distance 4
}

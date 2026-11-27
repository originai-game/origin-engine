type particles
version 1
---
# MIT Origin Engine. The forward flame tongue of sustained fire: one
# tongue per 0.1 s cycle, in step with muzzle-auto's loop.
maxParticles 8
shape cone 6 0.02
rate 0
burst 0 6
duration 0.1
looping true
lifetime 0.05 0.09
speed 4 7
size 0.07 0.12
heightOverLife 0 3 1 1.5
sizeOverLife 0 1 0.6 0.8 1 0
align velocity 0.04
gravity 0 0 0
blend additive
colorOverLife {
  key 0 3 2.2 1.2 1
  key 0.3 2.4 1.2 0.35 1
  key 1 0.6 0.12 0.02 0
}

type particles
version 1
---
# MIT Origin Engine. Motes drawn in to a point from all round it.
maxParticles 64
shape sphere 0.8 surface
rate 40
duration 2
looping true
lifetime 0.5 0.8
speed 0
size 0.012 0.025
gravity 0 0 0
attractor 0 0 0 7
drag 0.5
space local
align velocity 0.03
blend additive
colorOverLife {
  key 0 0 0 0 0
  key 0.3 0.6 1.2 2.6 1
  key 1 1 1.8 3.2 1
}

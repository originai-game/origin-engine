type particles
version 1
---
# MIT Origin Engine. A firework's burst: stars thrown out as a sphere,
# slowing, drooping, each drawing a tail.
maxParticles 100
shape sphere 0.1
spread 180
rate 0
burst 0 90
duration 0.05
looping false
lifetime 1.4 2.2
speed 7 11
size 0.1 0.16
gravity 0 -2.5 0
drag 1.6
trail 0.4 0.6
blend additive
colorOverLife {
  key 0 3 2.6 1.6 1
  key 0.3 2.6 1.6 0.5 1
  key 0.8 1.6 0.5 0.12 0.8
  key 1 0.3 0.05 0.01 0
}

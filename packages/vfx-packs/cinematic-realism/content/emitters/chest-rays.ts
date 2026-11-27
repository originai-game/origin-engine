type particles
version 1
---
# MIT Origin Engine. Light let out of an opened chest: thin upright rays
# that stand and fade.
maxParticles 10
shape box 0.25 0 0.15
rate 0
burst 0 8
duration 0.05
looping false
lifetime 0.6 1
speed 0
size 0.04 0.07
heightOverLife 0 2 0.3 16 1 10
gravity 0 0 0
align upright
blend additive
colorOverLife {
  key 0 2 1.5 0.6 0
  key 0.15 2 1.5 0.6 0.5
  key 1 0.6 0.4 0.1 0
}

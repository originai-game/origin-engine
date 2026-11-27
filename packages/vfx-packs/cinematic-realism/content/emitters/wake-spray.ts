type particles
version 1
---
# MIT Origin Engine. Droplets a hull or a propeller throws up; gone where
# they land.
maxParticles 96
shape cone 50 0.4
rate 80
duration 2
looping true
lifetime 0.4 0.8
speed 1.5 3.4
size 0.016 0.036
align velocity 0.02
gravity 0 -9.8 0
drag 0.3
blend alpha
color 0.88 0.94 1
colorOverLife {
  key 0 1 1 1 0.8
  key 0.8 1 1 1 0.6
  key 1 1 1 1 0
}
collision 0 0 depth die

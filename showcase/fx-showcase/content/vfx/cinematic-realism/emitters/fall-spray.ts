type particles
version 1
---
# MIT Origin Engine. Drops thrown up and out from the foot of a fall.
maxParticles 160
shape cone 60 0.8
rate 120
duration 2
looping true
lifetime 0.5 1
speed 2 4.5
size 0.012 0.03
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

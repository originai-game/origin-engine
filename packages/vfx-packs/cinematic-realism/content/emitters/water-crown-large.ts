type particles
version 1
---
# MIT Origin Engine. The sheet of drops thrown out around a blast's
# column.
maxParticles 200
shape cone 35 0.8
rate 0
burst 0.02 120
burst 0.12 60
duration 0.2
looping false
lifetime 1 2
speed 6 12
size 0.02 0.05
align velocity 0.02
gravity 0 -9.8 0
drag 0.25
blend alpha
color 0.88 0.94 1
colorOverLife {
  key 0 1 1 1 0.85
  key 0.8 1 1 1 0.65
  key 1 1 1 1 0
}
collision 0 0 depth die

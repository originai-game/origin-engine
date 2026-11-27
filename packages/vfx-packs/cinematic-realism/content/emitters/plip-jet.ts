type particles
version 1
---
# MIT Origin Engine. The few drops a small object throws straight up out
# of water.
maxParticles 16
shape cone 12 0.02
rate 0
burst 0 12
duration 0.05
looping false
lifetime 0.35 0.7
speed 1.5 3
size 0.01 0.022
align velocity 0.02
gravity 0 -9.8 0
drag 0.3
blend alpha
color 0.88 0.94 1
colorOverLife {
  key 0 1 1 1 0.85
  key 0.8 1 1 1 0.65
  key 1 1 1 1 0
}
collision 0 0 depth die

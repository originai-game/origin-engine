type particles
version 1
---
# MIT Origin Engine. Bright discs thrown up in a short fountain; they
# spin, fall and settle.
maxParticles 24
shape cone 28 0.05
rate 40
duration 0.5
looping false
lifetime 1.4 1.8
speed 3 5
size 0.04 0.05
heightOverLife 0 0.16 1 0.16
rotation 0 360
angularVelocity -900 900
gravity 0 -9.8 0
drag 0.2
render mesh sphere
lit true
roughness 0.3
blend alpha
color 0.95 0.72 0.2
colorOverLife {
  key 0 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0.35 0.5 depth

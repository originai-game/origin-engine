type particles
version 1
---
# MIT Origin Engine. A spent brass case: thrown from the port spinning, it
# bounces and lies.
maxParticles 1
shape cone 10 0.005
rate 0
burst 0 1
duration 0.05
looping false
lifetime 2
speed 2.2 3
size 0.012
heightOverLife 0 3.6 1 3.6
rotation 0 360
angularVelocity 500 1100
gravity 0 -9.8 0
drag 0.2
render mesh box
lit true
roughness 0.35
blend alpha
color 0.8 0.58 0.22
colorOverLife {
  key 0 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0.45 0.5 depth

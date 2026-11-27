type particles
version 1
---
# MIT Origin Engine. Shards of glass: clear, sharp, catching the light as
# they tumble.
maxParticles 24
shape cone 50 0.04
rate 0
burst 0 20
duration 0.05
looping false
lifetime 0.8 1.4
speed 1.5 5
size 0.01 0.03
rotation 0 360
angularVelocity -800 800
gravity 0 -9.8 0
drag 0.3
render mesh tetrahedron
lit true
roughness 0.06
blend alpha
color 0.78 0.88 0.92 0.55
colorOverLife {
  key 0 1 1 1 1
  key 0.85 1 1 1 1
  key 1 1 1 1 0
}
collision 0.3 0.4 depth

type particles
version 1
---
# MIT Origin Engine. A pane's large shards: flat blades a hand long that
# drop almost where they stood and skid on landing.
maxParticles 26
shape box 0.5 0.5 0.02
spread 60
rate 0
burst 0 22
duration 0.05
looping false
lifetime 1.4 2
speed 0.4 2.4
size 0.07 0.17
rotation 0 360
angularVelocity -360 360
gravity 0 -9.8 0
drag 0.15
render mesh tetrahedron
heightOverLife 0 0.1 1 0.1
lit true
roughness 0.05
blend alpha
color 0.8 0.9 0.94 0.6
colorOverLife {
  key 0 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0.25 0.5 depth

type particles
version 1
---
# MIT Origin Engine. The small pieces between a pane's shards: thrown
# further, tumbling fast, scattered wide on the ground.
maxParticles 84
shape box 0.5 0.5 0.02
spread 80
rate 0
burst 0 76
duration 0.05
looping false
lifetime 1 1.7
speed 0.8 3.6
size 0.015 0.045
rotation 0 360
angularVelocity -700 700
gravity 0 -9.8 0
drag 0.25
render mesh tetrahedron
heightOverLife 0 0.3 1 0.3
lit true
roughness 0.06
blend alpha
color 0.82 0.9 0.94 0.6
colorOverLife {
  key 0 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0.3 0.4 depth

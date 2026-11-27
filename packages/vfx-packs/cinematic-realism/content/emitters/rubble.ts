type particles
version 1
---
# MIT Origin Engine. Blocks of broken masonry: heavy, tumbling slowly,
# hardly bouncing.
maxParticles 20
shape sphere 0.4
spread 120
rate 0
burst 0 18
duration 0.05
looping false
lifetime 2 3
speed 1.5 5
size 0.07 0.22
heightOverLife 0 0.55 1 0.55
rotation 0 360
angularVelocity -200 200
gravity 0 -9.8 0
drag 0.2
render mesh box
lit true
roughness 0.95
blend alpha
color 0.45 0.44 0.42
colorOverLife {
  key 0 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0.2 0.8 depth

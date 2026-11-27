type particles
version 1
---
# MIT Origin Engine. Pebbles a wheel flings up; they fall and stop where
# they land.
maxParticles 40
shape cone 50 0.1
rate 18
duration 2
looping true
lifetime 0.6 1.1
speed 2 4.5
size 0.008 0.02
rotation 0 360
angularVelocity -600 600
gravity 0 -9.8 0
drag 0.3
render mesh tetrahedron
lit true
roughness 0.9
blend alpha
color 0.2 0.18 0.15
colorOverLife {
  key 0 1 1 1 1
  key 0.85 1 1 1 1
  key 1 1 1 1 0
}
collision 0.2 0.7 depth die

type particles
version 1
---
# MIT Origin Engine. The glowing gas an afterburner leaves in the world
# behind a moving craft.
maxParticles 96
shape cone 7 0.1
rate 50
duration 2
looping true
lifetime 0.5 1
speed 3 5
size 0.3 0.45
sizeOverLife 0 0.6 1 2.4
gravity 0 0 0
drag 1.5
blend additive
colorOverLife {
  key 0 0.8 0.4 0.12 0.4
  key 0.4 0.35 0.14 0.04 0.25
  key 1 0.05 0.02 0.01 0
}

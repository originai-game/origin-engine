type particles
version 1
---
# MIT Origin Engine. The cooler plume around the jet, left in the world
# so a moving craft draws a fading wake.
maxParticles 96
shape cone 9 0.06
rate 60
duration 2
looping true
lifetime 0.4 0.8
speed 1.5 2.5
size 0.1 0.16
sizeOverLife 0 0.6 1 2
gravity 0 0 0
drag 1.5
blend additive
colorOverLife {
  key 0 0.25 0.4 0.9 0.5
  key 0.4 0.12 0.22 0.55 0.3
  key 1 0.02 0.04 0.12 0
}

type particles
version 1
---
# MIT Origin Engine. An afterburner's flame, carried with the nozzle: blue
# at the root, orange down its length.
maxParticles 128
shape cone 3 0.07
rate 200
duration 2
looping true
lifetime 0.14 0.24
speed 10 15
size 0.13 0.19
sizeOverLife 0 0.9 0.4 1 1 0.3
heightOverLife 0 2.4 1 1
align velocity 0.02
gravity 0 0 0
space local
blend additive
colorOverLife {
  key 0 0.5 0.7 1.3 1
  key 0.15 1.1 0.75 0.4 0.9
  key 0.55 0.7 0.28 0.07 0.5
  key 1 0.1 0.02 0.005 0
}

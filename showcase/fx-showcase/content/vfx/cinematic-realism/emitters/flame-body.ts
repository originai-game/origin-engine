type particles
version 1
---
# MIT Origin Engine. Flames clinging to a burning figure.
maxParticles 12
shape box 0.22 0.55 0.18
rate 7
duration 2
looping true
lifetime 0.7 1.1
speed 0
size 0.5 0.8
sizeOverLife 0 0.8 0.5 1 1 0.9
gravity 0 0 0
align upright
space local
texture "vfx/cinematic-realism/textures/flames.png"
sheet 8 4 32 fps 20 blend
blend additive
color 1.4 1.4 1.4
softDistance 0.1
colorOverLife {
  key 0 1 1 1 0
  key 0.2 1 1 1 1
  key 0.75 1 1 1 1
  key 1 1 1 1 0
}

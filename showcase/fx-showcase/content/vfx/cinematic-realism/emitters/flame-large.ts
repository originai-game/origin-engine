type particles
version 1
---
# MIT Origin Engine. The body of a large fire: flames two metres and more,
# each standing a second.
maxParticles 16
shape sphere 0.6
rate 9
duration 2
looping true
lifetime 1 1.5
speed 0
size 1.8 2.6
sizeOverLife 0 0.8 0.5 1 1 0.9
gravity 0 0 0
align upright
texture "vfx/cinematic-realism/textures/flames.png"
sheet 8 4 32 fps 20 blend
blend additive
color 0.95 0.95 0.95
softDistance 0.5
colorOverLife {
  key 0 1 1 1 0
  key 0.2 1 1 1 1
  key 0.75 1 1 1 1
  key 1 1 1 1 0
}

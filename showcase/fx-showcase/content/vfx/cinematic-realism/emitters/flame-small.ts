type particles
version 1
---
# MIT Origin Engine. A fire about half a metre high: a few flames standing
# on its base, each fading in and out.
maxParticles 8
shape sphere 0.1
rate 5
duration 2
looping true
lifetime 0.9 1.3
speed 0
size 0.5 0.7
sizeOverLife 0 0.8 0.5 1 1 0.9
gravity 0 0 0
align upright
texture "vfx/cinematic-realism/textures/flames.png"
sheet 8 4 32 fps 20 blend
blend additive
color 1.05 1.05 1.05
softDistance 0.15
colorOverLife {
  key 0 1 1 1 0
  key 0.2 1 1 1 1
  key 0.75 1 1 1 1
  key 1 1 1 1 0
}

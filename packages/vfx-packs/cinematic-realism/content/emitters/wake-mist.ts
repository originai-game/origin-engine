type particles
version 1
---
# MIT Origin Engine. Fine spray hanging low over churned water.
maxParticles 24
shape cone 60 0.4
rate 8
duration 2
looping true
lifetime 1 1.8
speed 0.3 0.8
size 0.5 0.8
sizeOverLife 0 0.5 1 2
rotation 0 360
gravity 0 0.05 0
drag 2
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 0.95 1.08 1.2
softDistance 0.3
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.25
  key 1 1 1 1 0
}

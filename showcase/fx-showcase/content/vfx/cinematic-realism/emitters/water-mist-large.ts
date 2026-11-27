type particles
version 1
---
# MIT Origin Engine. Spray hanging over the water after the column has
# fallen.
maxParticles 12
shape cone 60 1
rate 0
burst 0.2 10
duration 0.3
looping false
lifetime 2.5 4
speed 0.6 1.6
size 2 3
sizeOverLife 0 0.5 1 1.8
rotation 0 360
gravity 0 0.05 0
drag 1.5
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 0.95 1.08 1.2
softDistance 0.6
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.35
  key 1 1 1 1 0
}

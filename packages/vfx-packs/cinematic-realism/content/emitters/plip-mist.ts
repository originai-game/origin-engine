type particles
version 1
---
# MIT Origin Engine. A breath of spray over a small splash.
maxParticles 3
shape cone 40 0.03
rate 0
burst 0 3
duration 0.05
looping false
lifetime 0.4 0.7
speed 0.2 0.5
size 0.12 0.2
sizeOverLife 0 0.4 1 2
rotation 0 360
gravity 0 0.05 0
drag 2
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 0.8 0.88 0.92
softDistance 0.1
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.3
  key 1 1 1 1 0
}

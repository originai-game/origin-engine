type particles
version 1
---
# MIT Origin Engine. A patch of foam where water was disturbed.
maxParticles 3
shape cone 85 0.05
rate 0
burst 0 3
duration 0.05
looping false
lifetime 0.8 1.2
speed 0.1 0.3
size 0.18 0.3
sizeOverLife 0 0.5 1 1.8
rotation 0 360
gravity 0 0 0
drag 2.5
align facing 0 1 0
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 1.15 1.2 1.25
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.5
  key 1 1 1 1 0
}

type particles
version 1
---
# MIT Origin Engine. White water spreading flat behind a hull.
maxParticles 110
shape box 0.5 0 0.15
spread 90
rate 36
duration 2
looping true
lifetime 2 3.2
speed 0.2 0.6
size 0.55 0.9
sizeOverLife 0 0.5 1 2.6
rotation 0 360
angularVelocity -20 20
gravity 0 0 0
drag 1.5
align facing 0 1 0
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 1.8 1.86 1.92
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.6
  key 0.6 1 1 1 0.34
  key 1 1 1 1 0
}

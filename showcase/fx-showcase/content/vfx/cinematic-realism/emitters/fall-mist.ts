type particles
version 1
---
# MIT Origin Engine. Mist boiling up where falling water lands.
maxParticles 48
shape box 1 0.2 0.4
spread 50
rate 14
duration 2
looping true
lifetime 2 3.5
speed 0.6 1.6
size 1 1.6
sizeOverLife 0 0.5 1 2.2
rotation 0 360
angularVelocity -10 10
gravity 0 0.1 0
drag 1.2
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 0.95 1.08 1.2
softDistance 0.5
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.3
  key 0.6 1 1 1 0.2
  key 1 1 1 1 0
}

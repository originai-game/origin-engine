type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. A road
# engine's exhaust: thin grey fumes that rise and spread.
maxParticles 48
shape cone 12 0.02
rate 20
duration 2
looping true
lifetime 1 1.8
speed 0.6 1.2
size 0.1 0.16
sizeOverLife 0 0.6 1 4
rotation 0 360
gravity 0 0.25 0
drag 2
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.95 0.95 0.98
softDistance 0.1
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.2
  key 1 1 1 1 0
}

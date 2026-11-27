type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The little
# dust a footfall raises.
maxParticles 8
shape cone 75 0.06
rate 0
burst 0 7
duration 0.05
looping false
lifetime 0.7 1.2
speed 0.4 1
size 0.16 0.26
sizeOverLife 0 0.5 1 2.2
rotation 0 360
gravity 0 0.05 0
drag 3
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1 0.92 0.78
softDistance 0.1
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.42
  key 1 1 1 1 0
}

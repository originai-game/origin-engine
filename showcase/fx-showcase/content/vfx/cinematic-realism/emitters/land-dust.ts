type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The ring of
# dust a hard landing drives out along the ground.
maxParticles 12
shape cone 86 0.15
rate 0
burst 0 10
duration 0.05
looping false
lifetime 0.8 1.3
speed 1.5 2.5
size 0.3 0.5
sizeOverLife 0 0.5 1 2
rotation 0 360
gravity 0 0.08 0
drag 3.5
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1 0.92 0.78
softDistance 0.2
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.45
  key 0.6 1 1 1 0.25
  key 1 1 1 1 0
}

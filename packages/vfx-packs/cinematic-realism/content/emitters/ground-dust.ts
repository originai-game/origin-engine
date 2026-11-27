type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The skirt of
# dust a ground blast pushes outward along the surface.
maxParticles 16
shape cone 88 0.6
rate 0
burst 0 14
duration 0.05
looping false
lifetime 2 3
speed 3 5
size 0.7 1.1
sizeOverLife 0 0.5 0.4 1.2 1 1.8
gravity 0 0.08 0
drag 2.6
wind 1
rotation 0 360
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.62 0.57 0.5
colorOverLife {
  key 0 1 1 1 0
  key 0.06 1 1 1 0.85
  key 0.6 1 1 1 0.45
  key 1 1 1 1 0
}
softDistance 0.4

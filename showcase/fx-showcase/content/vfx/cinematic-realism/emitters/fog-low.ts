type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. Fog lying in
# a hollow, hardly moving.
maxParticles 140
shape box 3 0.1 3
spread 90
rate 11
duration 2
looping true
lifetime 8 12
speed 0.05 0.2
size 3.2 4.8
rotation 0 360
angularVelocity -3 3
gravity 0 0 0
drag 0.5
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1.3 1.34 1.4
softDistance 1
colorOverLife {
  key 0 1 1 1 0
  key 0.2 1 1 1 0.075
  key 0.8 1 1 1 0.075
  key 1 1 1 1 0
}

type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The grey
# cloud broken masonry raises.
maxParticles 9
shape sphere 0.4
spread 180
rate 0
burst 0 8
duration 0.05
looping false
lifetime 2 3
speed 0.6 1.6
size 0.8 1.2
sizeOverLife 0 0.5 1 2
rotation 0 360
angularVelocity -12 12
gravity 0 0.08 0
drag 2
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.8 0.78 0.74
softDistance 0.4
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.6
  key 0.6 1 1 1 0.35
  key 1 1 1 1 0
}

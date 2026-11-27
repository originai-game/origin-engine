type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The dust out
# of a broken crate.
maxParticles 6
shape sphere 0.3
spread 180
rate 0
burst 0 5
duration 0.05
looping false
lifetime 1 1.6
speed 0.4 1.2
size 0.5 0.8
sizeOverLife 0 0.5 1 2
rotation 0 360
gravity 0 0.05 0
drag 2.5
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.9 0.8 0.62
softDistance 0.2
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.45
  key 1 1 1 1 0
}

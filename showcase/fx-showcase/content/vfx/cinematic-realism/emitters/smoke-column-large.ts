type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The column
# over a large blast: it climbs for seconds and hangs.
maxParticles 24
shape sphere 1.6
rate 0
burst 0.5 20
duration 0.6
looping false
lifetime 5 7
speed 0.4 1
size 3 4.5
sizeOverLife 0 0.6 1 1.7
rotation 0 360
angularVelocity -8 8
gravity 0 0.8 0
drag 0.6
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.28 0.26 0.25
softDistance 1
colorOverLife {
  key 0 1 1 1 0
  key 0.12 1 1 1 0.8
  key 0.6 1 1 1 0.5
  key 1 1 1 1 0
}

type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. Smoke from a
# wreck or a distant fire: a column that climbs, leans with the wind and
# spreads.
maxParticles 96
shape sphere 0.4
rate 9
duration 2
looping true
lifetime 6 9
speed 0.6 1.2
size 1.2 1.8
sizeOverLife 0 0.5 1 4
rotation 0 360
angularVelocity -8 8
gravity 0 0.5 0
drag 0.4
wind 1
noise 0.3 0.3 0.2
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.15 0.14 0.14
softDistance 0.8
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.8
  key 0.6 1 1 1 0.5
  key 1 1 1 1 0
}

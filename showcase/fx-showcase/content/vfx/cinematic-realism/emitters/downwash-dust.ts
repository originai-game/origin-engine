type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. Dust driven
# outwards along the ground under a rotor or a landing jet.
maxParticles 400
shape cone 89 1.8
rate 150
duration 2
looping true
lifetime 1.6 2.6
speed 4 7
size 1 1.5
sizeOverLife 0 0.5 1 2.4
rotation 0 360
angularVelocity -20 20
gravity 0 0.02 0
drag 1.6
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1.4 1.34 1.22
softDistance 0.9
colorOverLife {
  key 0 1 1 1 0
  key 0.25 1 1 1 0.13
  key 0.6 1 1 1 0.085
  key 1 1 1 1 0
}

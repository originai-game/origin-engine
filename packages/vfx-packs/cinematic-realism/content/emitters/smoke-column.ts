type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The dark
# column that rises and lingers after the fireball burns out.
maxParticles 16
shape sphere 0.6
rate 0
burst 0.35 12
duration 0.4
looping false
lifetime 2.6 3.6
speed 0.2 0.5
size 1.2 1.8
sizeOverLife 0 0.6 1 1.6
gravity 0 0.6 0
drag 0.9
wind 1
rotation 0 360
angularVelocity -12 12
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.3 0.28 0.27
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.75
  key 0.6 1 1 1 0.5
  key 1 1 1 1 0
}
softDistance 0.5

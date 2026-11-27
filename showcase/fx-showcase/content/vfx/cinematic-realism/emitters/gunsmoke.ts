type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. Pale gun
# smoke drifting from the barrel after the flash.
maxParticles 6
shape cone 18 0.03
rate 0
burst 0 5
duration 0.05
looping false
lifetime 0.8 1.3
speed 0.6 1.4
size 0.16 0.26
sizeOverLife 0 0.5 1 2.6
rotation 0 360
angularVelocity -40 40
gravity 0 0.15 0
drag 3
wind 1
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1 0.98 0.95
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.32
  key 1 1 1 1 0
}
softDistance 0.1

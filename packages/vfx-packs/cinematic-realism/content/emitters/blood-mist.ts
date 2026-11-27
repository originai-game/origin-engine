type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The fine red
# mist a bullet or blade throws from a body: a quick puff out of the wound
# that spreads, darkens and settles in under a second.
maxParticles 8
shape cone 30 0.02
rate 0
burst 0 6
duration 0.05
looping false
lifetime 0.35 0.7
speed 1 2.6
size 0.12 0.2
sizeOverLife 0 0.4 0.3 1.6 1 2.4
rotation 0 360
angularVelocity -60 60
gravity 0 -0.6 0
drag 6
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.42 0.03 0.025
softDistance 0.08
colorOverLife {
  key 0 1 1 1 0
  key 0.06 1 1 1 0.75
  key 0.4 0.7 0.7 0.7 0.4
  key 1 0.5 0.5 0.5 0
}

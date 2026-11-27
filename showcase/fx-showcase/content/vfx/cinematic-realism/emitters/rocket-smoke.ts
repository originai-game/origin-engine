type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. The trail a
# rocket leaves in the world: lit warm where it is born, then grey,
# widening and drifting with the wind.
maxParticles 400
shape sphere 0.04
spread 180
rate 140
duration 2
looping true
lifetime 1.8 2.8
speed 0.1 0.5
size 0.4 0.6
sizeOverLife 0 0.7 0.2 1.2 1 3
rotation 0 360
angularVelocity -25 25
gravity 0 0.12 0
drag 1.5
wind 1
noise 0.25 0.8 0.3
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.82 0.8 0.78
softDistance 0.2
colorOverLife {
  key 0 1.6 1.2 0.8 0
  key 0.04 1.5 1.15 0.8 0.85
  key 0.15 1 1 1 0.7
  key 0.6 1 1 1 0.35
  key 1 1 1 1 0
}

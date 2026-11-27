type particles
version 1
---
# MIT Origin Engine. Foam boiling outwards on the surface round a blast.
maxParticles 18
shape cone 86 0.8
rate 0
burst 0 16
duration 0.05
looping false
lifetime 2.4 3.4
speed 1.5 3
size 1.4 2.2
sizeOverLife 0 0.5 1 1.8
rotation 0 360
angularVelocity -20 20
gravity 0 0 0
drag 1.5
align facing 0 1 0
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 1.15 1.2 1.25
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.6
  key 0.6 1 1 1 0.3
  key 1 1 1 1 0
}

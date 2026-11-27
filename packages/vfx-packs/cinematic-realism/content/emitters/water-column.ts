type particles
version 1
---
# MIT Origin Engine. The white column an underwater blast drives up; it
# stalls and falls back as rain.
maxParticles 30
shape cone 8 0.5
rate 0
burst 0 16
burst 0.1 10
duration 0.2
looping false
lifetime 1.6 2.4
speed 9 15
size 0.9 1.5
sizeOverLife 0 0.6 1 2.2
rotation 0 360
angularVelocity -30 30
gravity 0 -9.8 0
drag 0.6
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 1.1 1.14 1.18
softDistance 0.5
colorOverLife {
  key 0 1 1 1 0
  key 0.06 1 1 1 0.9
  key 0.6 1 1 1 0.6
  key 1 1 1 1 0
}

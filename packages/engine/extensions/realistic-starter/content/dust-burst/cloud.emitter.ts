type particles
version 1
---
# The billowing cloud of a collapse: heavy lit dust rolling out low and hanging.
maxParticles 32
shape box 0.8 0.3 0.8
spread 80
rate 0
burst 0 22
duration 0.05
looping false
lifetime 2.2 3.4
speed 1 3
size 0.9 1.5
sizeOverLife 0 0.5 1 2.2
rotation 0 360
angularVelocity -20 20
gravity 0 0.05 0
drag 1.8
wind 1
forceFields 1
color 0.8 0.72 0.62
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.9
  key 0.6 1.05 1.05 1.05 0.65
  key 1 1.1 1.1 1.1 0
}
texture "vfx/realistic-starter/textures/smoke.png"
sheet 2 2 4 life 1 blend
blend alpha
lit true
softDistance 0.6

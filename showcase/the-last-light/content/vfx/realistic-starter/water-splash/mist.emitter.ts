type particles
version 1
---
# The fine mist left hanging over it, drifting with the air.
maxParticles 12
shape sphere 0.3
spread 180
rate 0
burst 0.05 8
duration 0.05
looping false
lifetime 1 1.5
speed 0.3 0.9
size 0.4 0.65
sizeOverLife 0 0.6 1 1.9
rotation 0 360
angularVelocity -20 20
gravity 0 0 0
drag 2
wind 1
color 0.88 0.92 0.96
colorOverLife {
  key 0 1 1 1 0
  key 0.15 1 1 1 0.28
  key 1 1 1 1 0
}
texture "vfx/realistic-starter/textures/smoke.png"
sheet 2 2 4 life 1 blend
blend alpha
lit true
softDistance 0.3

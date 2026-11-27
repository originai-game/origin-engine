type particles
version 1
---
# The smoke that follows: dark, lit, rising and spreading with the air.
maxParticles 24
shape sphere 0.5
spread 70
rate 0
burst 0.08 18
duration 0.1
looping false
lifetime 2.2 3.3
speed 0.6 2.4
size 1 1.5
sizeOverLife 0 0.5 1 2.3
rotation 0 360
angularVelocity -20 20
gravity 0 0.9 0
drag 1.2
wind 1
forceFields 1
color 0.42 0.4 0.38
colorOverLife {
  key 0 0.5 0.45 0.4 0
  key 0.12 1 1 1 0.95
  key 0.6 1.3 1.3 1.3 0.75
  key 1 1.6 1.6 1.6 0
}
texture "vfx/realistic-starter/textures/smoke.png"
sheet 2 2 4 life 1 blend
blend alpha
lit true
softDistance 0.6

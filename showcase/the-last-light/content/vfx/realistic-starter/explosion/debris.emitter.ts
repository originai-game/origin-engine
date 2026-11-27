type particles
version 1
---
# Earth and stone thrown up: solid lit chunks that tumble, land and bounce.
maxParticles 24
shape cone 55 0.3
spread 15
rate 0
burst 0 16
duration 0.05
looping false
lifetime 1.3 1.9
speed 4 9
size 0.06 0.15
rotation 0 360
angularVelocity -540 540
gravity 0 -9.8 0
collision 0.25 0.6
color 0.32 0.28 0.24
colorOverLife {
  key 0 1 1 1 1
  key 0.85 1 1 1 1
  key 1 1 1 1 0
}
texture "vfx/realistic-starter/textures/dirt.png"
sheet 3 1 3 random
blend alpha
lit true

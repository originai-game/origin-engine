type particles
version 1
---
# The fireball: large bright billows that swell, rise, and cool from yellow to dark red.
maxParticles 24
shape sphere 0.35
spread 180
rate 0
burst 0 16
duration 0.05
looping false
lifetime 0.45 0.85
speed 1.5 3.5
size 0.9 1.4
sizeOverLife 0 0.45 0.25 1 1 1.35
rotation 0 360
angularVelocity -60 60
gravity 0 2 0
drag 4
color 1 1 1
colorOverLife {
  key 0 9 6 3.2 1
  key 0.25 6 2.6 0.8 1
  key 0.6 2.2 0.6 0.12 0.8
  key 1 0.5 0.1 0.02 0
}
texture "vfx/realistic-starter/textures/fire.png"
sheet 2 1 2 random
blend additive
softDistance 0.4

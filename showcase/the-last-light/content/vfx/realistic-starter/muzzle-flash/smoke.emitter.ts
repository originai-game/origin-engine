type particles
version 1
---
# Powder smoke: a grey wisp that rolls forward, slows and drifts with the air.
maxParticles 8
shape cone 20 0.03
spread 10
rate 0
burst 0.02 4
duration 0.05
looping false
lifetime 0.7 1.1
speed 0.8 2
size 0.1 0.16
sizeOverLife 0 0.6 1 2.6
rotation 0 360
angularVelocity -40 40
gravity 0 0.25 0
drag 3
wind 1
color 0.75 0.73 0.7
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.55
  key 1 1 1 1 0
}
texture "vfx/realistic-starter/textures/smoke.png"
sheet 2 2 4 life 1 blend
blend alpha
lit true
softDistance 0.15

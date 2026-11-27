type particles
version 1
---
# MIT Origin Engine. A flamethrower's stream: burning fuel thrown forward,
# swelling as it slows and lifting at its end.
maxParticles 160
shape cone 5 0.03
rate 150
duration 2
looping true
lifetime 0.55 0.85
speed 9 13
size 0.2 0.28
sizeOverLife 0 0.6 0.3 2 1 5
rotation 0 360
angularVelocity -60 60
gravity 0 1.6 0
drag 2.2
texture "vfx/cinematic-realism/textures/flames.png"
sheet 8 4 32 random
blend additive
softDistance 0.2
colorOverLife {
  key 0 2.6 2.2 1.6 0.9
  key 0.2 2.2 1.5 0.7 0.9
  key 0.6 1.6 0.7 0.2 0.6
  key 1 0.4 0.08 0.02 0
}

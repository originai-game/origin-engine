type particles
version 1
---
# Sparks off a hard surface: fast, bright, streaked, bouncing once and going out.
maxParticles 32
shape cone 45 0.01
spread 15
rate 0
burst 0 22
duration 0.05
looping false
lifetime 0.15 0.45
speed 3 9
size 0.01 0.018
gravity 0 -9.8 0
drag 1.2
collision 0.3 0.3
align velocity 0.04
color 6 3 1.1
colorOverLife {
  key 0 1.4 1.2 1 1
  key 0.5 1 0.55 0.2 1
  key 1 0.8 0.25 0.05 0
}
blend additive

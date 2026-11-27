type particles
version 1
---
# The flames: tall tongues standing upright, licking up and cooling at their tips.
maxParticles 32
shape cone 8 0.06
spread 0
rate 34
lifetime 0.35 0.6
speed 0.5 0.9
size 0.14 0.22
sizeOverLife 0 0.6 0.3 1 1 0.35
heightOverLife 0 1.3 1 1.9
gravity 0 1.2 0
noise 0.35 2 1.2
drag 0.5
wind 0.4
align upright
color 1 1 1
colorOverLife {
  key 0 1.6 0.9 0.4 0
  key 0.12 2.4 1.25 0.45 1
  key 0.55 1.6 0.6 0.14 0.8
  key 1 0.8 0.15 0.02 0
}
texture "vfx/realistic-starter/textures/flame.png"
sheet 2 2 4 life 1 blend
blend additive

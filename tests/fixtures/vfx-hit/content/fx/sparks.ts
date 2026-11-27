type particles
version 1
---
# The hit's sparks: one burst of hot streaks thrown out and down, bouncing
# off the ground, white to orange as they cool.
maxParticles 128
shape sphere 0.1
spread 180
rate 0
burst 0 70
duration 0.1
looping false
lifetime 0.25 0.65
speed 4 10
size 0.05 0.1
sizeOverLife 0 1 1 0.3
colorOverLife {
  key 0 8 6 4 1
  key 0.3 6 2.4 0.6 1
  key 1 2 0.4 0.05 0
}
align velocity 0.035
texture "textures/spark.png"
blend additive
gravity 0 -9 0
drag 1.2
collision 0.35 0.5 depth

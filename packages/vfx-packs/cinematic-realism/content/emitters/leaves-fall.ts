type particles
version 1
---
# MIT Origin Engine. Dry leaves coming down over a few metres of ground,
# turning as they fall.
maxParticles 32
shape box 2 0.2 2
spread 180
rate 3.5
duration 2
looping true
lifetime 5 7
speed 0.1 0.4
size 0.08 0.13
rotation 0 360
angularVelocity -120 120
gravity 0 -0.6 0
drag 0.9
wind 1
noise 0.6 0.5 0.3
texture "vfx/cinematic-realism/textures/leaves.png"
sheet 2 2 4 random
blend alpha
lit true
colorOverLife {
  key 0 1 1 1 0
  key 0.05 1 1 1 1
  key 0.9 1 1 1 1
  key 1 1 1 1 0
}
collision 0 1 depth die

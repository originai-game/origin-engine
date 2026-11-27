type particles
version 1
---
# MIT Origin Engine. Molten sparks thrown by an electrical arc: white-hot
# at the break, cooling orange as they fall and bounce, a spray with each
# discharge of electric-sparks' 2 s loop.
maxParticles 64
shape cone 70 0.04
rate 0
burst 0 16
burst 0.6 6
burst 1.1 18
duration 2
looping true
lifetime 0.4 1
speed 1.8 4.5
size 0.012 0.024
gravity 0 -9.8 0
drag 0.8
align velocity 0.03
trail 0.07 0.45 only
blend additive
colorOverLife {
  key 0 3 3 3.4 1
  key 0.15 2.6 1.6 0.6 1
  key 0.6 1.2 0.4 0.08 1
  key 1 0.3 0.06 0.01 0
}
collision 0.35 0.3 depth

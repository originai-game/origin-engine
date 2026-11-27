type particles
version 1
---
# MIT Origin Engine. Sparks spat now and then from damage or a fire,
# falling and bouncing.
maxParticles 32
shape cone 60 0.05
rate 0
burst 0.3 5
burst 1.4 7
duration 2
looping true
lifetime 0.4 0.9
speed 1.5 4
size 0.01 0.02
gravity 0 -9.8 0
drag 0.8
align velocity 0.025
trail 0.06 0.35 only
blend additive
colorOverLife {
  key 0 2 1.2 0.45 1
  key 0.5 1.2 0.4 0.08 1
  key 1 0.3 0.06 0.01 0
}
collision 0.35 0.3 depth

type particles
version 1
---
# MIT Origin Engine. A body going to ash: dark flakes shed from its whole
# height, lifted and carried off.
maxParticles 160
shape box 0.28 0.8 0.2
spread 180
rate 110
duration 1.2
looping false
rateOverTime 0 0.4 0.5 1 1 0.6
lifetime 1 2
speed 0.1 0.5
size 0.03 0.06
rotation 0 360
angularVelocity -200 200
gravity 0 0.5 0
drag 1
wind 1
noise 0.5 0.8 0.4
blend alpha
color 0.08 0.075 0.07
colorOverLife {
  key 0 1 1 1 0
  key 0.1 1 1 1 0.9
  key 0.7 1 1 1 0.7
  key 1 1 1 1 0
}

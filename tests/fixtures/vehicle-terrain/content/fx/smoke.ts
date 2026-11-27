type particles
version 1
---
# Tyre smoke at a rear wheel while the car slides: a pale puff that drifts
# up and spreads, lit by the sun.
maxParticles 256
shape sphere 0.15
spread 60
rate 90
lifetime 0.8 1.4
speed 0.4 1.2
size 0.3 0.6
sizeOverLife 0 0.5 1 2.4
colorOverLife {
  key 0 0.82 0.8 0.78 0
  key 0.15 0.82 0.8 0.78 0.8
  key 1 0.85 0.84 0.82 0
}
gravity 0 0.5 0
drag 1.5
blend alpha
lit true
softDistance 0.3

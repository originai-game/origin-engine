type particles
version 1
---
# MIT Origin Engine. Heat haze along a jet or a flame stream: billboards
# that only bend the scene behind them.
maxParticles 24
shape cone 6 0.05
rate 14
duration 2
looping true
lifetime 0.5 0.9
speed 3 6
size 0.4 0.7
sizeOverLife 0 0.5 1 1.8
gravity 0 0.8 0
drag 1.5
blend alpha
distortion 0.01
color 1 1 1 0
colorOverLife {
  key 0 1 1 1 0
  key 0.25 1 1 1 1
  key 1 1 1 1 0
}

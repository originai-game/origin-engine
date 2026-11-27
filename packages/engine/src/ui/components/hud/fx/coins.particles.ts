type particles
version 1
---
# A purchase's coins for a `<Particles>` node: coins thrown up from the
# centre in a fountain, spinning (their height swings) as they fall. Grey
# shading: the node's ink (text-gold) gilds them.
maxParticles 40
shape cone 30 4
spread 0
rate 0
burst 0 26
duration 0.1
looping false
lifetime 1.3 1.7
speed 420 620
size 18 26
heightOverLife 0 1 0.14 0.2 0.28 1 0.42 0.2 0.56 1 0.7 0.2 0.84 1 1 0.2
rotation -20 20
gravity 0 -1300 0
texture "@styles/shared/fx/coin.png"
colorOverLife {
  key 0 1 1 1 1
  key 0.8 1 1 1 1
  key 1 1 1 1 0
}

type particles
version 1
---
# A rarity reveal's rays for a `<Particles>` node: long soft beams
# crossing at the centre, turning slowly as they flare and fade. White:
# the node's ink (text-<rarity>) colours them.
maxParticles 12
shape point
rate 0
burst 0 6
duration 0.1
looping false
lifetime 2.0 2.4
speed 0
size 30 40
heightOverLife 0 3 0.25 12 1 12
rotation 0 180
angularVelocity 20 34
gravity 0 0 0
texture "@styles/shared/fx/ray.png"
colorOverLife {
  key 0 1 1 1 0
  key 0.2 1 1 1 0.85
  key 0.7 1 1 1 0.55
  key 1 1 1 1 0
}

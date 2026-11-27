type particles
version 1
---
# A reveal's charge for a `<Particles>` node: glints drawn in from a
# ring around the centre, gathering as they shrink. White: the ink
# colours them.
maxParticles 32
shape sphere 150 surface
spread 0
rate 0
burst 0 22
duration 0.1
looping false
lifetime 0.24
speed -640 -560
size 8 14
sizeOverLife 0 0.4 1 1
gravity 0 0 0
texture "@styles/shared/fx/glint.png"
colorOverLife {
  key 0 1 1 1 0
  key 0.3 1 1 1 1
  key 1 1 1 1 1
}

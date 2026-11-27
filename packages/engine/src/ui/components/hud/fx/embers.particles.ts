type particles
version 1
---
# Embers for a `<Particles>` node (1 unit a CSS px, +y up): round glints
# rising from along the node's box, shrinking as they fade. White: the
# node's ink (text-<colour>) colours it.
maxParticles 48
shape box 1 1 0
spread 28
rate 14
burst 0 10
duration 1.6
looping false
lifetime 0.9 1.2
speed 40 110
size 3 5
sizeOverLife 0 1 1 0.2
round true
gravity 0 30 0
drag 0.4
colorOverLife {
  key 0 1 1 1 1
  key 0.5 1 1 1 1
  key 1 1 1 1 0
}

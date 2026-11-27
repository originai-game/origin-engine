type particles
version 1
---
# A shock ring for a `<Particles>` node: one ring of light growing from
# the centre and fading as it widens. White: the node's ink colours it.
maxParticles 2
shape point
rate 0
burst 0 1
duration 0.1
looping false
lifetime 0.7
speed 0
size 300
sizeOverLife 0 0.15 1 1
gravity 0 0 0
texture "@styles/shared/fx/ring.png"
colorOverLife {
  key 0 1 1 1 0.9
  key 1 1 1 1 0
}

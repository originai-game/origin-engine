type particles
version 1
---
# A firework's shell for a `<Particles>` node (1 unit a CSS px, +y up):
# one burst of round sparks from the node's centre, falling as they
# spread and fade. White: the node's ink (text-<colour>) colours it.
maxParticles 32
shape sphere 2
spread 180
rate 0
burst 0 28
duration 0.1
looping false
lifetime 1.1 1.5
speed 90 170
size 5 8
sizeOverLife 0 1 1 0.4
round true
gravity 0 -70 0
drag 0.9
colorOverLife {
  key 0 1 1 1 1
  key 0.6 1 1 1 1
  key 1 1 1 1 0
}

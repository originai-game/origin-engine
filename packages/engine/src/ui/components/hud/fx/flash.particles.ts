type particles
version 1
---
# A flash for a `<Particles>` node: one soft glow swelling at the centre
# and fading fast, the light a reward or a reveal starts with. White: the
# node's ink colours it.
maxParticles 2
shape point
rate 0
burst 0 1
duration 0.1
looping false
lifetime 0.5
speed 0
size 260
sizeOverLife 0 0.4 1 1
gravity 0 0 0
texture "@styles/shared/fx/glint.png"
colorOverLife {
  key 0 1 1 1 0.9
  key 1 1 1 1 0
}

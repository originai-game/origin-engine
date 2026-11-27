type particles
version 1
---
# A rarity reveal's shards for a `<Particles>` node: faceted crystal
# shards burst from the centre, tumbling, slowing and falling. Grey
# shading: the node's ink (text-<rarity>) colours them.
maxParticles 40
shape sphere 8
spread 180
rate 0
burst 0 26
duration 0.1
looping false
lifetime 1.0 1.5
speed 220 520
size 10 16
rotation 0 360
angularVelocity -240 240
gravity 0 -220 0
drag 2.0
texture "@styles/shared/fx/shard.png"
colorOverLife {
  key 0 1 1 1 1
  key 0.75 1 1 1 1
  key 1 1 1 1 0
}

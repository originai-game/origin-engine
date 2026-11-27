type particles
version 1
---
# CC0 Thomas Iché / Unity Labs Explosion02; MIT composition. A large
# blast's fireball: three bodies of fire rolling into each other, then two
# more through them.
maxParticles 5
shape sphere 1.2
rate 0
burst 0 3
burst 0.12 2
duration 0.2
looping false
lifetime 3.4 4.2
speed 0.2 0.8
size 4.2 5.4
rotation 0 360
gravity 0 0.5 0
texture "vfx/cinematic-realism/textures/explosion.png"
sheet 5 5 25 life 1 blend
blend alpha
softDistance 0.6
colorOverLife {
  key 0 0.65 0.65 0.65 1
  key 0.2 0.85 0.85 0.85 1
  key 0.7 1 1 1 0.9
  key 1 1 1 1 0
}

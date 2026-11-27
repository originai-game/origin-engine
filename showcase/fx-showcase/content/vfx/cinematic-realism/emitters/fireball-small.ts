type particles
version 1
---
# CC0 Thomas Iché / Unity Labs Explosion02; MIT composition. A grenade's
# fireball: brief, about a metre and a half across.
maxParticles 1
shape point
rate 0
burst 0 1
duration 0.05
looping false
lifetime 1.6
speed 0
size 1.6
gravity 0 0.3 0
texture "vfx/cinematic-realism/textures/explosion.png"
sheet 5 5 25 life 1 blend
blend alpha
softDistance 0.2
colorOverLife {
  key 0 0.65 0.65 0.65 1
  key 0.2 0.85 0.85 0.85 1
  key 0.7 1 1 1 0.9
  key 1 1 1 1 0
}

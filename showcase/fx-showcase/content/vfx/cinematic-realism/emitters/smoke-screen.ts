type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01; MIT composition. A smoke
# grenade's cloud: fed for eight seconds, it stands and drifts for as long
# again.
maxParticles 160
shape cone 70 0.15
rate 14
duration 8
looping false
rateOverTime 0 1 0.8 1 1 0.3
lifetime 7 10
speed 0.4 1.2
size 1.2 1.8
sizeOverLife 0 0.4 0.2 1 1 2.5
rotation 0 360
angularVelocity -8 8
gravity 0 0.08 0
drag 1.2
wind 1
noise 0.2 0.4 0.2
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
lit true
color 0.85 0.86 0.88
softDistance 0.6
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.85
  key 0.7 1 1 1 0.7
  key 1 1 1 1 0
}

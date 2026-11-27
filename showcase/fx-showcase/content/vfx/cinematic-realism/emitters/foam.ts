type particles
version 1
---
# CC0 Thomas Iché / Unity Labs DiscSmoke01 as surface foam; MIT
# composition. White water lying flat on the surface, spreading and thinning.
maxParticles 10
shape cone 85 0.12
rate 0
burst 0 8
duration 0.05
looping false
lifetime 1.1 1.6
speed 0.4 0.9
size 0.35 0.6
sizeOverLife 0 0.5 1 1.8
rotation 0 360
angularVelocity -30 30
gravity 0 0 0
drag 2.5
align facing 0 1 0
texture "vfx/cinematic-realism/textures/dust.png"
sheet 16 4 64 life 1 blend
blend alpha
color 1.15 1.2 1.25
colorOverLife {
  key 0 1 1 1 0
  key 0.08 1 1 1 0.55
  key 0.6 1 1 1 0.3
  key 1 1 1 1 0
}

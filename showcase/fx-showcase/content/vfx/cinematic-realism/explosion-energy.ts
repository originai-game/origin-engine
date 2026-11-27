type vfx
version 2
---
duration 6
impact 0
param mark 1
mesh sphere {
  life 0.45
  offset 0 0.8 0
  effect "vfx/cinematic-realism/energy-shell.ogt"
  scale curve 0 0.3 0.3 1.6 1 2.4
  color 0.5 1.1 2.8
}
mesh ring {
  life 0.5
  offset 0 0.05 0
  rotate -90 0 0
  segments 64
  inner 0.8
  colors 1 1 1 0 1 1 1 1 1 1 1 0
  effect "vfx/cinematic-realism/shield-ripple.ogt"
  scale curve 0 0.4 0.4 2.6 1 3.6
  color 0.5 1 2.4
}
emitter "vfx/cinematic-realism/emitters/energy-flecks.ogt" {
  offset 0 0.8 0
}
emitter "vfx/cinematic-realism/emitters/warp-shock.ogt" {
  offset 0 0.8 0
}
glow {
  life 0.2
  offset 0 0.8 0
  size curve 0 1 0.3 3 1 1.5
  color 1.2 1.8 3.2 0.8
}
light {
  life 0.5
  offset 0 0.9 0
  color 0.45 0.7 1
  intensity curve 0 12 0.2 6 1 0
  distance 9
}
shake {
  trauma 0.35
}
decal {
  at 0.05
  until 6
  fade 3
  when mark
  texture "vfx/cinematic-realism/textures/scorch.png"
  normal "vfx/cinematic-realism/textures/scorch-normal.png"
  size 3 0.8 3
  color 0.7 0.75 0.85 0.8
  roughness 0.9
  angleFade 80
  turn 0 360
}
decal {
  at 0.05
  life 3
  fade 2.6
  when mark
  texture "vfx/cinematic-realism/textures/embers.png"
  size 1.6 0.8 1.6
  color 0.05 0.05 0.06 1
  emissive 0.6 1.4 3.2
  angleFade 80
  turn 0 360
}
sound "base/vfx/cinematic-realism/audio/zap" {
  volume 0.9
  playbackRate 0.6 0.7
}
sound "base/vfx/cinematic-realism/audio/blast-small" {
  volume 0.5
  playbackRate 0.8 0.9
}

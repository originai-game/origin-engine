type vfx
version 2
---
duration 2
impact 0.2
mesh ring {
  life 0.9
  offset 0 0.02 0
  rotate -90 0 0
  segments 48
  inner 0.86
  colors 1 1 1 0 1 1 1 1 1 1 1 0
  effect "vfx/cinematic-realism/shield-ripple.ogt"
  scale curve 0 0.3 0.5 0.7 1 0.8
  color 0.5 1.6 0.9
}
emitter "vfx/cinematic-realism/emitters/rise-flecks.ogt" {
  offset 0 0.05 0
}
glow {
  life 1.2
  offset 0 1 0
  size curve 0 0.6 0.3 1.6 1 1
  color 0.4 1.2 0.7 0.3
}
light {
  life 1.4
  offset 0 1 0
  color 0.55 1 0.7
  intensity curve 0 0 0.2 1.4 1 0
  distance 4
}
sound "base/vfx/cinematic-realism/audio/chime" {
  playbackRate 0.75 0.8
}

type vfx
version 2
---
duration 2
loop true
beam {
  at 0
  life 0.1
  to 0.32 0.16 0.1
  segments 10
  jitter 0.05
  width 0.022
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
beam {
  at 0.04
  life 0.07
  to -0.2 0.28 0.12
  segments 10
  jitter 0.05
  width 0.018
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
beam {
  at 0.1
  life 0.08
  to 0.12 0.3 -0.26
  segments 10
  jitter 0.05
  width 0.02
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
beam {
  at 0.6
  life 0.06
  to -0.16 0.18 -0.1
  segments 10
  jitter 0.05
  width 0.015
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
beam {
  at 1.1
  life 0.1
  to 0.18 0.26 -0.28
  segments 10
  jitter 0.05
  width 0.022
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
beam {
  at 1.15
  life 0.07
  to -0.3 0.14 0.08
  segments 10
  jitter 0.05
  width 0.018
  effect "vfx/cinematic-realism/beam.ogt"
  color 2.6 3.8 6.4 1
}
emitter "vfx/cinematic-realism/emitters/arc-sparks.ogt" {
}
glow {
  at 0
  life 0.16
  offset 0 0.08 0
  size curve 0 0.4 1 0.08
  color 1.4 2 3.4 0.7
}
glow {
  at 0.6
  life 0.08
  offset 0 0.08 0
  size curve 0 0.22 1 0.08
  color 1.4 2 3.4 0.7
}
glow {
  at 1.1
  life 0.16
  offset 0 0.08 0
  size curve 0 0.4 1 0.08
  color 1.4 2 3.4 0.7
}
light {
  at 0
  life 0.18
  offset 0 0.3 0
  color 0.6 0.8 1
  intensity curve 0 1.2 0.3 0.3 0.5 0.96 1 0
  distance 3.5
}
light {
  at 0.6
  life 0.08
  offset 0 0.3 0
  color 0.6 0.8 1
  intensity curve 0 0.6 0.3 0.15 0.5 0.48 1 0
  distance 3.5
}
light {
  at 1.1
  life 0.18
  offset 0 0.3 0
  color 0.6 0.8 1
  intensity curve 0 1.2 0.3 0.3 0.5 0.96 1 0
  distance 3.5
}
sound "base/vfx/cinematic-realism/audio/zap" {
  at 0
  volume 0.35
  playbackRate 1.3 1.6
}
sound "base/vfx/cinematic-realism/audio/zap" {
  at 0.6
  volume 0.2
  playbackRate 1.6 1.9
}
sound "base/vfx/cinematic-realism/audio/zap" {
  at 1.1
  volume 0.35
  playbackRate 1.4 1.7
}

type vfx
version 2
---
duration 3.2
impact 0.8
emitter "vfx/cinematic-realism/emitters/fw-rise.ogt" {
  move 0 14 0
  until 0.8
}
glow {
  life 0.8
  move 0 14 0
  size 0.35
  color 2.4 1.6 0.7 0.7
}
emitter "vfx/cinematic-realism/emitters/fw-burst.ogt" {
  at 0.8
  offset 0 11.2 0
}
emitter "vfx/cinematic-realism/emitters/fw-glitter.ogt" {
  at 1.3
  offset 0 10.6 0
}
glow {
  at 0.8
  life 0.2
  offset 0 11.2 0
  size curve 0 2 1 5
  color 2.6 2 1.2 0.7
}
light {
  at 0.8
  life 0.6
  offset 0 11.2 0
  color 1 0.8 0.5
  intensity curve 0 20 1 0
  distance 30
}
sound "base/vfx/cinematic-realism/audio/whoosh" {
  volume 0.5
  playbackRate 1.6 1.8
}
sound "base/vfx/cinematic-realism/audio/blast-small" {
  at 0.8
  volume 0.6
  playbackRate 1.3 1.5
}

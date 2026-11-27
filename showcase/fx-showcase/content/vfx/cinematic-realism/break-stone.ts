type vfx
version 2
---
duration 3.5
impact 0
emitter "vfx/cinematic-realism/emitters/rubble.ogt" {
  offset 0 0.5 0
}
emitter "vfx/cinematic-realism/emitters/chips.ogt" {
  offset 0 0.5 0
}
emitter "vfx/cinematic-realism/emitters/rubble-dust.ogt" {
  offset 0 0.4 0
}
shake {
  trauma 0.25
}
sound "base/vfx/cinematic-realism/audio/crack" {
  playbackRate 0.55 0.7
}
sound "base/vfx/cinematic-realism/audio/thud" {
  volume 1.2
  playbackRate 0.6 0.75
}

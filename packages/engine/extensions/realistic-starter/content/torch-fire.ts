type vfx
version 2
---
# A torch's or brazier's fire, realistic and looping: upright flames, embers, thin smoke and a flickering light. It stands on its origin: spawn it at the top of the torch.
duration 4
loop true
emitter "vfx/realistic-starter/torch-fire/flames.emitter.ogt" {
  at 0
}
emitter "vfx/realistic-starter/torch-fire/embers.emitter.ogt" {
  at 0
  offset 0 0.15 0
}
emitter "vfx/realistic-starter/torch-fire/smoke.emitter.ogt" {
  at 0
  offset 0 0.45 0
}
light {
  at 0
  life 4
  offset 0 0.3 0
  color 1 0.6 0.26
  intensity curve 0 0.8 0.15 0.62 0.3 0.92 0.45 0.66 0.6 0.84 0.75 0.58 0.9 0.76 1 0.8
  distance 3
}

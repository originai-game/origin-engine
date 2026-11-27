type sfx
version 1
---
# MIT Origin Engine. A round on something hard: a bright tick and a short
# ring.
layer noise {
  highpass 2500
  attack 0.001
  sustain 0.004
  decay 0.04
  gain 0.6
}
layer sine {
  freq 950
  freq_end 320
  attack 0.001
  sustain 0.004
  decay 0.035
  gain 0.3
}
layer sine {
  freq 3100
  attack 0.001
  sustain 0.01
  decay 0.12
  gain 0.08
}

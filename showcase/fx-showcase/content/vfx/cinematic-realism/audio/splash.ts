type sfx
version 1
---
# MIT Origin Engine. Water thrown up and falling back.
layer noise {
  lowpass 3200
  lp_slide -0.3
  highpass 350
  attack 0.012
  sustain 0.08
  decay 0.6
  gain 0.6
}
layer sine {
  freq 190
  freq_end 85
  attack 0.004
  sustain 0.02
  decay 0.12
  gain 0.35
}
layer noise {
  highpass 2800
  delay 0.12
  attack 0.05
  sustain 0.1
  decay 0.5
  gain 0.15
}

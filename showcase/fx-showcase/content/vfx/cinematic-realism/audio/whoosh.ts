type sfx
version 1
---
# MIT Origin Engine. Air torn by something fast: swelling, then gone.
layer noise {
  lowpass 700
  lp_slide 0.6
  highpass 120
  attack 0.22
  sustain 0.08
  decay 0.45
  gain 0.7
}
layer sine {
  freq 60
  freq_end 140
  attack 0.2
  sustain 0.05
  decay 0.3
  gain 0.3
}

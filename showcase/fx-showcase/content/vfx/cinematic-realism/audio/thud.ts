type sfx
version 1
---
# MIT Origin Engine. A soft hit: earth, cloth or timber taking a blow.
layer sine {
  freq 125
  freq_end 58
  attack 0.002
  sustain 0.015
  decay 0.09
  gain 0.7
}
layer noise {
  lowpass 750
  attack 0.002
  sustain 0.01
  decay 0.11
  gain 0.5
}

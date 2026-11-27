type sfx
version 1
---
# MIT Origin Engine. An energy discharge: a falling buzz and a hiss.
layer saw {
  freq 1500
  freq_end 180
  fm_freq 90
  fm_index 2
  attack 0.001
  sustain 0.02
  decay 0.16
  lowpass 5000
  gain 0.4
}
layer noise {
  highpass 3200
  attack 0.001
  sustain 0.01
  decay 0.09
  gain 0.3
}
layer sine {
  freq 90
  freq_end 45
  attack 0.002
  sustain 0.02
  decay 0.12
  gain 0.4
}

type sfx
version 1
---
# MIT Origin Engine. A rifle shot: the crack, a short chest thump and the
# report rolling off.
layer noise {
  lowpass 5200
  highpass 500
  attack 0.001
  sustain 0.012
  decay 0.16
  punch 0.8
  gain 0.8
}
layer sine {
  freq 160
  freq_end 50
  attack 0.001
  sustain 0.01
  decay 0.1
  drive 2
  gain 0.7
}
layer noise {
  lowpass 1400
  delay 0.02
  attack 0.005
  sustain 0.03
  decay 0.45
  gain 0.25
}

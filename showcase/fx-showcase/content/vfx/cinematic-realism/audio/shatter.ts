type sfx
version 1
---
# MIT Origin Engine. Glass breaking: the crack, then shards ringing as
# they fall.
layer noise {
  highpass 3800
  attack 0.001
  sustain 0.02
  decay 0.25
  punch 0.8
  gain 0.6
}
layer sine {
  freq 4200
  delay 0.04
  attack 0.001
  sustain 0.005
  decay 0.09
  gain 0.15
}
layer sine {
  freq 5300
  delay 0.11
  attack 0.001
  sustain 0.005
  decay 0.07
  gain 0.12
}
layer sine {
  freq 3400
  delay 0.19
  attack 0.001
  sustain 0.005
  decay 0.1
  gain 0.1
}
layer noise {
  highpass 5000
  delay 0.1
  attack 0.02
  sustain 0.1
  decay 0.4
  gain 0.15
}

type flipbook
version 1
---
# The knight: three generated GIFs (512x512, magenta ground) keyed, held on
# their feet and cut to one cell. The idle is drawn larger than the others.
source "hero_idle.gif" {
  clip "idle"
  scale 0.89
}
source "hero_run.gif" {
  clip "run"
  event 1 "step"
  event 5 "step"
}
source "hero_attack.gif" {
  clip "attack"
  mode once
  event 3 "hit"
}
key auto 0.1 holes
align feet
scale 0.25
filter nearest

type model
version 1
---
# The wind-bent hawthorn over the crag: its crown combed inland by the
# sea wind. Its colours are pale: og.Model.tint gives them the season's
# (src/climate.ts).
plant "hawthorn" {
  kind oak
  seed 17
  force -1 0.25 0.35 0.05
  barkColor 0.3 0.24 0.18
  trunk {
    length 2.6
    radius 0.24
    gnarliness 0.16
  }
  leaves {
    size 0.1
    color 0.6 0.6 0.3
    color2 0.85 0.75 0.3
  }
}

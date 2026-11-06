type model
version 1
---
# A glass pane, 1.6 x 1.2 m and 1 cm thick, standing on its foot: struck, it
# cracks radially and in rings from the blow and leaves no shard intact.
material "glass" {
  color 0.84 0.95 0.9
  roughness 0.02
  transmission 1
  ior 1.5
  thickness 0.01
  doubleSided true
}
part "pane" {
  material "glass"
  position 0 0.6 0
  boxGeometry {
    width 1.6
    height 1.2
    depth 0.01
  }
}
fracture {
  style glass
  chunks 25
  rings 3
  seed 9
  children 0
  impact 0 0.8 0 0.25
  interior {
    color 0.55 0.85 0.8
    roughness 0.05
  }
}

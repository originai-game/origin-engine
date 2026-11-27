type effect
version 1
---
# MIT Origin Engine. The crown sheet of a splash: a thin water wall torn
# into vertical fingers by stretched noise erosion as it rises and falls.
blend alpha
doubleSided true
noise {
  tile 12 1.2
}
erosion curve 0 0.3 0.4 0.5 1 0.95
erosionSoftness 0.12
fresnel 2.5 1.2
color 0.85 0.92 1
alpha curve 0 0.5 0.5 0.35 1 0
softDepth 0.03

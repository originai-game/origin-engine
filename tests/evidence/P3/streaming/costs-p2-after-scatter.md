P2 feature costs on Apple M4 Pro (metal), 1280x720, best of 5 runs of 100 frames (`og-gfx --costs-p2`).

| Scene | ms | without the effect | Δ | draws |
|---|---|---|---|---|
| `terrain-near`: P2: terrain from 1.7 m (2 km, 4 layers, clipmap) | 1.091 | 0.862 | +0.229 | 38 |
| `terrain-mid`: P2: terrain from 120 m | 1.526 | 1.381 | +0.145 | 36 |
| `terrain-far`: P2: terrain from 1.5 km | 1.995 | 1.409 | +0.585 | 24 |
| `terrain-mid`: P2: terrain casting shadows (its shadow passes) | 1.501 | 1.880 | -0.380 | 36 |
| `water-planar`: P2: planar reflection (half resolution) | 3.747 | 2.934 | +0.812 | 49 |
| `water-shadows`: P2: shadows received on water | 2.416 | 3.174 | -0.758 | 44 |
| `water-underwater`: P2: caustics on the bed | 2.932 | 2.834 | +0.098 | 37 |
| `water-underwater`: P2: underwater view (fog, the surface from below; vs no water) | 1.623 | 1.758 | -0.135 | 37 |
| `terrain-grass`: P2: grass scatter (36,000 tufts, shadowed) | 2.499 | 2.043 | +0.456 | 201 |

Feature costs on Apple M4 Pro (metal), 1280x720, best of 2 sweeps x 5 runs of 100 frames (`og-gfx --costs`).

| Variant | gltf-reference ms | Δ | draws | spheres-20k ms | Δ | draws |
|---|---|---|---|---|---|---|
| features off (CSM on) | 1.679 |  | 113 | 8.014 |  | 7 |
| no shadows at all | 1.097 | -0.582 | 40 | 4.695 | -3.319 | 4 |
| TAA | 1.941 | +0.262 | 114 | 8.146 | +0.132 | 8 |
| MSAA 4x | 2.167 | +0.488 | 113 | 11.919 | +3.905 | 7 |
| bloom | 2.173 | +0.494 | 124 | 7.550 | -0.464 | 18 |
| auto exposure | 1.874 | +0.195 | 116 | 7.944 | -0.070 | 10 |
| 4 point/spot lights | 1.817 | +0.138 | 113 | 8.650 | +0.636 | 7 |
| 4 lights, 2 shadowed | 2.953 | +1.275 | 308 | 8.637 | +0.623 | 20 |
| all (TAA, bloom, auto exposure, 4 lights / 2 shadowed) | 3.921 | +2.242 | 323 | 9.982 | +1.967 | 35 |

`sprites-20k` (20,000 sprites moved and resubmitted every frame, orthographic): 0.861 ms per frame, 4 draws (3 sprite draws).
`text-msdf` (131 glyphs): 0.259 ms per frame, 2 draws.


Measured 2026-10-18 under load average 15-25 (other agents building); compare with tests/evidence/M3/feature-costs.md for the M3 numbers. The deltas keep the M3 shape; absolute times are 5-20 % higher, attributable to load (og-gfx --all shows RMSE 0 for every frame, so the work drawn is unchanged).

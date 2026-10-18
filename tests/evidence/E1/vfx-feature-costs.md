Feature costs on Apple M4 Pro (metal), 1280x720, best of 2 sweeps x 5 runs of 100 frames (`og-gfx --costs`).

| Variant | gltf-reference ms | Δ | draws | spheres-20k ms | Δ | draws |
|---|---|---|---|---|---|---|
| features off (CSM on) | 1.391 |  | 113 | 8.211 |  | 7 |
| no shadows at all | 0.857 | -0.533 | 40 | 4.880 | -3.331 | 4 |
| TAA | 1.970 | +0.579 | 114 | 8.511 | +0.300 | 8 |
| MSAA 4x | 2.022 | +0.631 | 113 | 12.632 | +4.421 | 7 |
| bloom | 1.779 | +0.389 | 124 | 8.337 | +0.126 | 18 |
| auto exposure | 1.506 | +0.115 | 116 | 7.924 | -0.287 | 10 |
| 4 point/spot lights | 1.531 | +0.140 | 113 | 8.390 | +0.179 | 7 |
| 4 lights, 2 shadowed | 2.598 | +1.207 | 308 | 8.706 | +0.495 | 20 |
| E1: AO (GTAO) | 3.221 | +1.830 | 153 | 13.514 | +5.303 | 11 |
| E1: AO (SSAO) | 2.907 | +1.516 | 153 | 13.014 | +4.803 | 11 |
| E1: outline (scene-wide) | 2.542 | +1.151 | 152 | 12.371 | +4.160 | 10 |
| E1: depth of field (low) | 2.608 | +1.217 | 154 | 12.511 | +4.300 | 12 |
| E1: depth of field (high) | 2.617 | +1.227 | 154 | 12.702 | +4.491 | 12 |
| E1: motion blur | 2.473 | +1.083 | 152 | 12.640 | +4.429 | 10 |
| E1: god rays | 2.415 | +1.024 | 153 | 12.137 | +3.926 | 11 |
| E1: grading (LUT, vignette, aberration, grain) | 1.450 | +0.060 | 113 | 6.908 | -1.303 | 7 |
| E1: 8 decals | 1.789 | +0.399 | 113 | 9.105 | +0.894 | 7 |
| E1: toon + rim on every material | 1.634 | +0.244 | 113 | 7.482 | -0.729 | 7 |
| all (TAA, bloom, auto exposure, 4 lights / 2 shadowed) | 2.973 | +1.583 | 323 | 9.080 | +0.869 | 35 |

`sprites-20k` (20,000 sprites moved and resubmitted every frame, orthographic): 1.189 ms per frame, 4 draws (3 sprite draws).

E1 effect scenes (each against itself with the effect removed):

| Scene | ms | without the effect | Δ | draws |
|---|---|---|---|---|
| `particles`: particles (GPU, 6 systems) | 1.248 | 1.176 | +0.071 | 58 |
| `particles-cpu`: particles (CPU path, 6 systems) | 1.490 | 1.440 | +0.050 | 58 |
| `fx-2d`: 2D lights + soft shadows | 0.960 | 0.975 | -0.015 | 23 |
| `fx-2d`: 2D glow buffer | 2.269 | 0.423 | +1.846 | 23 |
| `trails-beams`: trails, ribbon, beams | 0.863 | 1.308 | -0.446 | 30 |
| `decals`: 5 decals | 1.083 | 1.411 | -0.328 | 24 |
| `stylised`: outlines (prepass + pass) | 1.767 | 1.038 | +0.729 | 38 |
| `skinning`: 5 skinned instances (vs none) | 0.411 | 0.392 | +0.019 | 10 |
`text-msdf` (131 glyphs): 0.147 ms per frame, 2 draws.

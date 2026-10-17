Feature costs on Apple M4 Pro (metal), 1280x720, best of 2 sweeps x 5 runs of 100 frames (`og-gfx --costs`).

| Variant | gltf-reference ms | Δ | draws | spheres-20k ms | Δ | draws |
|---|---|---|---|---|---|---|
| features off (CSM on) | 1.434 |  | 113 | 6.836 |  | 7 |
| no shadows at all | 0.775 | -0.660 | 40 | 3.982 | -2.854 | 4 |
| TAA | 1.721 | +0.287 | 114 | 6.840 | +0.004 | 8 |
| MSAA 4x | 1.750 | +0.316 | 113 | 9.907 | +3.071 | 7 |
| bloom | 1.754 | +0.320 | 124 | 6.614 | -0.222 | 18 |
| auto exposure | 1.519 | +0.085 | 116 | 6.540 | -0.296 | 10 |
| 4 point/spot lights | 1.437 | +0.003 | 113 | 7.365 | +0.529 | 7 |
| 4 lights, 2 shadowed | 2.494 | +1.059 | 308 | 7.455 | +0.618 | 20 |
| all (TAA, bloom, auto exposure, 4 lights / 2 shadowed) | 3.743 | +2.309 | 323 | 8.700 | +1.863 | 35 |

`sprites-20k` (20,000 sprites moved and resubmitted every frame, orthographic): 1.055 ms per frame, 4 draws (3 sprite draws).
`text-msdf` (131 glyphs): 0.307 ms per frame, 2 draws.

## Against the M0 renderer

Interleaved A/B on the same machine and moment (`og-gfx gltf-reference spheres-20k --frames 200`, 9 rounds alternating the M0 binary built from ffd0e3d with glam `scalar-math` and the M3 binary; load average 18–90 from other agents and an iOS simulator):

```
base gltf-reference          median 1.587 min 1.454 (all: 1.454 1.494 1.555 1.577 1.587 1.595 1.602 1.706 1.716)
base spheres-20k             median 8.040 min 7.385 (all: 7.385 7.431 7.673 7.986 8.040 8.366 8.523 8.744 9.651)
new gltf-reference           median 1.581 min 1.429 (all: 1.429 1.503 1.515 1.569 1.581 1.595 1.636 1.705 1.747)
new spheres-20k              median 6.778 min 6.155 (all: 6.155 6.182 6.627 6.756 6.778 6.834 6.934 7.415 7.464)
```

The task's references were 1.6 ms and 7.8 ms (M0's README): `gltf-reference` is within ±10 % (1.58 ms median), `spheres-20k` is 13 % below 7.8 ms (6.78 ms) and 16 % below M0 measured alongside, because the three cascades draw fewer casters than M0's single map (casters culled to the camera slice swept towards the light: 969 vs 926×1 instances drawn on gltf, 16,719 vs 20,001 on spheres) and the main pass culls to the frustum.

## Reference frames across tiers (Metal vs vulkan-cpu)

| Scene | RMSE | pixels off by > 32/255 |
|---|---|---|
| bloom | 0.00128 | 0.000 % |
| cascades-200m | 0.00147 | 0.000 % |
| exposure | 0.00192 | 0.000 % |
| gltf-reference | 0.00441 | 0.006 % |
| point-spot-lights | 0.00135 | 0.000 % |
| showcase | 0.00490 | 0.030 % |
| sky-only | 0.00106 | 0.000 % |
| spheres-20k | 0.00675 | 0.127 % |
| sprites-20k | 0.00257 | 0.000 % |
| taa | 0.00134 | 0.000 % |
| text-msdf | 0.00364 | 0.006 % |

On lavapipe the 22 gfx tests pass against the frames captured in the same sandbox run with RMSE 0 (deterministic). Lavapipe frame times (E2B, 2 vCPU): gltf-reference 243 ms, sprites-20k 65 ms.

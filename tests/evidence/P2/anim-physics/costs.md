# Phase 2, animation and physics round: costs

Apple M4 Pro, 2026-10-19, with other agents' builds on the machine (load
average 18 to 44 during these runs): the numbers are upper bounds, the
ratios between rows the useful part.

## Physics, CPU per 60 Hz tick

`cargo run --release -p og-physics --example anim_physics_costs`: each scene
built fresh three times, run 30 ticks, then timed over 150 (the best of the
three), minus an empty world with a floor (3D 0.003 ms, 2D 0.001 ms).

| Cloth, lowered onto the floor | particles | ms per tick, no self-collision | ms per tick, self-collision (thickness = spacing, 2 substeps) |
|---|---|---|---|
| 12x12 | 144 | 0.045 | 0.266 |
| 24x24 | 576 | 0.218 | 1.200 |
| 32x32 | 1024 | 0.403 | 2.359 |
| 48x48 | 2304 | 1.024 | 4.558 |
| 64x64 | 4096 | 1.215 | 6.909 |

Self-collision costs about 5 to 6 times the plain cloth (two substeps, and
the spatial hash rebuilt and searched every solver pass). The fx-showcase
cloth station's 20 x 20 sheet is in the 12x12 to 24x24 range: about 0.5 ms.
Above 32 x 32 it is more than a tenth of a 16.7 ms frame; keep
self-colliding cloth to capes and sheets of a few hundred points.

| Fluids (2D, a 6 m basin) | particles | ms per tick |
|---|---|---|
| one fluid alone | 1000 | 1.243 |
| two of 500 in one group, layering (mixing 0) | 1000 | 1.324 |
| two of 500 in one group, mixing (1) | 1000 | 1.417 |
| four of 250 in one group | 1000 | 1.394 |
| eight of 125 alone | 1000 | 0.757 |
| eight of 125 in two groups of four | 1000 | 1.264 |

A group costs what one fluid of all its particles costs, plus 7 to 14 %
for the masses, the boundary tension and mixing. Fluids alone are cheaper
than one fluid of the same particles (smaller neighbour searches), so a
group is only worth it where the fluids touch.

| Ragdolls, knocked over on the floor | bones | ms per tick |
|---|---|---|
| 1 in 2D | 11 | 0.009 |
| 8 in 2D | 88 | 0.068 |
| 1 in 3D | 11 | 0.020 |
| 8 in 3D | 88 | 0.180 |

A 2D ragdoll costs about 40 % of the same rig in 3D.

## Retargeting

Retargeting runs once per clip and rig: at cook time for a model's
`animation { library }`, or on the first `og.anim.retarget` of a (clip,
source, target) at run time, then cached. It is not a per-tick cost.

## Rendering, GPU per frame

`og-gfx --costs-e5 --frames 60` on Metal at 1280 x 720, best of five runs,
four runs under the load above (Δ is the scene minus itself without its
metaballs):

| Scene | Δ ms, four runs |
|---|---|
| `fluid-2d`: two fluids, one field (unchanged path) | +0.249, +0.305, +0.484, +0.514 |
| `fluid-groups`: ten fluids in three fields, two groups of two | +0.264, +0.717, +1.034, +1.440 |

Each field of four fluids is one more splat pass and one more channel set
in the composite; ten fluids draw three fields, so they cost roughly two
to three times the two-fluid scene. The wide spread is the machine's load,
not the scenes.

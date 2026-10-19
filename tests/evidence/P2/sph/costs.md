# 3D fluid costs (owed row 29)

## CPU: the simulation (og-physics)

`cargo run --release -p og-physics --example fluid3d_costs` on the M4 Pro
(native aarch64) and the same scenes built for wasm32 and run under Node
v26.3.0; ms per 60 Hz tick, each the best of ten 30-tick windows after 120
ticks of settling. The machine was shared with other agents' builds (load
59 during the native run, 34 during the wasm one), so these are the least
disturbed windows, not a quiet machine's.

| Scene | particles | substeps | native | wasm32 |
|---|---|---|---|---|
| pool on a floor | 500 | 1 | 0.171 | 0.161 |
| pool on a floor | 500 | 2 | 0.268 | 0.285 |
| pool on a floor | 1,000 | 1 | 0.391 | 0.403 |
| pool on a floor | 1,000 | 2 | 0.603 | 0.634 |
| pool on a floor | 2,000 | 1 | 0.863 | 0.907 |
| pool on a floor | 2,000 | 2 | 1.826 | 1.778 |
| pool on a floor | 4,000 | 1 | 2.598 | 2.141 |
| pool on a floor | 4,000 | 2 | 4.158 | 4.335 |
| pool on a floor | 8,000 | 1 | 4.582 | 4.806 |
| pool on a floor | 8,000 | 2 | 9.780 | 9.346 |
| poured into a terrain bowl (piled deep) | 2,000 | 2 | 3.968 | — |
| a group of two fluids layering | 2 × 1,000 | 2 | 2.514 | — |
| a 600/s emitter (maxCount 2,000) into a sink | up to 2,000 | 2 | 1.902 | — |

The cost follows the particles and their neighbours: about linear in the
count, doubled by a second substep, and roughly doubled again where they
pile deep (the bowl), since each has more neighbours. A group of two
fluids costs about one fluid of all their particles plus 40 %. wasm32 is
within 10 % of native except 4,000 at one substep (18 % faster, a noisy
window).

## GPU: the screen-space surface

`./target/release/og-gfx --costs-p2`, the `fluid-3d` scene (two fluids,
2,534 particles, about a third of a 1280×720 frame), best of 5 runs of 10
frames, against the same scene with no fluid. The machine was shared with
other agents' gates (load average 27 to 98), so the absolute numbers vary
from run to run; the difference does not vary much.

| Device | run | ms | without the fluid | Δ | draws |
|---|---|---|---|---|---|
| Apple M4 Pro (metal) | 1 | 2.444 | 1.275 | +1.169 | 36 |
| Apple M4 Pro (metal) | 2 | 1.846 | 0.733 | +1.113 | 36 |
| Apple M4 Pro (metal) | 3 | 1.583 | 0.501 | +1.082 | 36 |
| lavapipe (vulkan-cpu, x86_64 under emulation) | 1 | 219.027 | 74.563 | +144.464 | 36 |

On a GPU the surface costs about 1.1 ms at this coverage whatever the particle
count. It grows with the pixels the fluid covers: two instanced sphere draws
(depth, thickness), two bilateral blur passes (12 taps each side, at most
24 px) and the composite in the main pass. The lavapipe row shows how costly the
passes are on a CPU rasteriser; it is not a budget for players.


# GPU culling threshold, measured again (`GPU_CULLING_MIN_INSTANCES`)

2026-10-20, branch `web-host-tools` (from 6a44eaf, with the round's
uncommitted og-lanes changes in the tree), Apple M4 Pro, macOS, Metal 3.
The machine was shared with other agents' builds and gates and with this
round's own wasm build: 1-minute load averages 20 to 33 (median 26 with
lanes at their default, 22 with lanes off). Read medians and in-round
ratios, not single runs.

The question (docs/phase-1.md, culling round, "Left"): `Culling::Auto` takes
the GPU path from 16,384 instances, a figure measured before the review
fixes cut the GPU path's fixed per-frame cost. Is it still the crossover?

## How

og-tests' perf-only city scenes (`crates/og-tests/src/culling_scenes.rs`)
now take `OG_GFX_CITY_SIDE=<n>`: an n × n city (n rounded up to even) instead
of 200 × 200, so the same scene runs at any instance count. The default is
unchanged (40,000 buildings).

    cargo build --release -p og-tests --bin og-gfx
    OG_GFX_CITY_SIDE=64 OG_GFX_CULLING=cpu target/release/og-gfx city-street --frames 300
    OG_GFX_CITY_SIDE=64 OG_GFX_CULLING=gpu target/release/og-gfx city-street --frames 300

`culling/interleave.sh ROUNDS FRAMES OUT [SIDES]` runs, per round, every
side and both scenes with the CPU path and the GPU path forced back to back
(the order alternating by round), and logs the load average at the start of
each round. Runs:

    tests/evidence/P4/web-host-tools/culling/interleave.sh 6 300 culling/rounds-a.txt                      # sides 16 32 46 64 90 128 180 256
    tests/evidence/P4/web-host-tools/culling/interleave.sh 6 300 culling/rounds-b.txt 16 32 40 46 54 64 76 90 128 180 256
    OG_LANES=off tests/evidence/P4/web-host-tools/culling/interleave.sh 6 300 culling/rounds-c-lanes-off.txt 16 32 40 46 54 64 76 90 128 180 256
    python3 culling/summarize.py culling/rounds-a.txt culling/rounds-b.txt   # summary-lanes-default.md
    python3 culling/summarize.py culling/rounds-c-lanes-off.txt               # summary-lanes-off.md

The measures are og-gfx's, as in the culling round
(`tests/evidence/P3/culling/NOTES.md`): `render` is `cpu=`, the wall time of
`Renderer::render` per frame (encode, cull, upload, submit), and `frame` is
`frame=`, that plus the wait for the GPU, both averaged over the 300 timed
frames of a run. The ratio is the median of the per-round GPU/CPU ratios;
under 1 the GPU path is cheaper.

## Lanes off (the cull runs serially: the web's single-threaded wasm, `OG_LANES=off`)

6 rounds, ms per frame (medians):

| Scene | Instances | render CPU | render GPU | GPU/CPU | frame CPU | frame GPU | GPU/CPU |
|---|---|---|---|---|---|---|---|
| street | 256 | 0.633 | 1.359 | 2.23 | 0.909 | 1.619 | 2.10 |
| street | 1,024 | 0.590 | 1.250 | 1.90 | 0.800 | 1.556 | 1.84 |
| street | 2,116 | 1.114 | 1.454 | 1.79 | 1.210 | 1.631 | 1.95 |
| street | 4,096 | 0.995 | 1.426 | 1.58 | 1.026 | 1.621 | 1.67 |
| street | 8,100 | 1.151 | 1.418 | 1.28 | 1.162 | 1.651 | 1.32 |
| street | 16,384 | 1.627 | 1.372 | 0.98 | 1.645 | 1.402 | 1.00 |
| street | 32,400 | 2.888 | 2.120 | 0.77 | 2.893 | 2.133 | 0.77 |
| street | 65,536 | 4.889 | 3.665 | 0.73 | 4.902 | 3.676 | 0.73 |
| aerial | 256 | 0.505 | 1.218 | 2.31 | 0.613 | 1.566 | 2.62 |
| aerial | 1,024 | 0.633 | 1.296 | 1.90 | 0.754 | 1.519 | 1.99 |
| aerial | 2,116 | 0.804 | 1.554 | 2.18 | 0.886 | 1.935 | 2.09 |
| aerial | 4,096 | 0.849 | 1.519 | 1.95 | 0.915 | 1.661 | 2.01 |
| aerial | 8,100 | 0.992 | 1.200 | 1.14 | 1.014 | 1.376 | 1.31 |
| aerial | 16,384 | 1.393 | 1.461 | 1.06 | 1.401 | 1.534 | 1.12 |
| aerial | 32,400 | 2.502 | 2.172 | 0.92 | 2.510 | 2.199 | 0.95 |
| aerial | 65,536 | 5.434 | 3.521 | 0.67 | 5.448 | 3.539 | 0.67 |

(1,600, 2,916 and 5,776 instances are in `culling/summary-lanes-off.md`;
they fall between their neighbours.) The GPU path's fixed cost is about
1.2 to 1.5 ms of `render` on this loaded machine, against 0.5 to 0.6 ms for
the CPU path on a small city. The two meet at about 16,000 instances on the
street (0.98 at 16,384) and between 16,384 and 32,400 from the air (1.06,
then 0.92); from 32,400 the GPU path is cheaper on both, by a quarter to a
third at 65,536.

## Lanes at their default (native: the cull splits across the pool from 2,048 bounds)

12 rounds (6 at 1,600, 2,916 and 5,776), ms per frame (medians):

| Scene | Instances | render CPU | render GPU | GPU/CPU | frame CPU | frame GPU | GPU/CPU |
|---|---|---|---|---|---|---|---|
| street | 256 | 0.702 | 1.391 | 1.89 | 0.827 | 1.586 | 1.76 |
| street | 1,024 | 0.728 | 1.423 | 1.83 | 0.881 | 1.530 | 1.77 |
| street | 1,600 | 1.024 | 1.679 | 1.60 | 1.222 | 1.872 | 1.46 |
| street | 2,116 | 1.703 | 1.499 | 1.16 | 1.712 | 1.626 | 1.29 |
| street | 4,096 | 2.201 | 1.411 | 0.74 | 2.211 | 1.526 | 0.84 |
| street | 8,100 | 3.153 | 1.613 | 0.59 | 3.167 | 1.720 | 0.60 |
| street | 16,384 | 4.733 | 2.018 | 0.50 | 4.745 | 2.058 | 0.50 |
| street | 65,536 | 13.178 | 4.745 | 0.57 | 13.189 | 4.759 | 0.57 |
| aerial | 256 | 0.600 | 1.346 | 2.16 | 0.712 | 1.560 | 2.27 |
| aerial | 1,600 | 0.703 | 1.654 | 2.28 | 0.889 | 1.968 | 2.28 |
| aerial | 2,116 | 3.171 | 1.627 | 0.69 | 3.179 | 1.675 | 0.70 |
| aerial | 4,096 | 1.587 | 1.268 | 0.85 | 1.599 | 1.406 | 0.95 |
| aerial | 8,100 | 1.792 | 1.527 | 0.79 | 1.800 | 1.692 | 0.87 |
| aerial | 16,384 | 7.202 | 2.503 | 0.42 | 7.213 | 2.519 | 0.45 |
| aerial | 65,536 | 11.733 | 5.256 | 0.57 | 11.751 | 5.267 | 0.57 |

(Every count: `culling/summary-lanes-default.md`.) Here the CPU path jumps
as soon as the frustum cull splits (from 2,048 bounds, `CULL_GRAIN` 1,024)
and is two to three times its lanes-off cost from then on (13.2 ms against
4.9 ms at 65,536 on the street): the pool hop the lanes round measured at
0.5 to 2.6 ms a call on a loaded machine, paid per culled view (camera,
three cascades), not culling work. The GPU path has no CPU cull to split, so
it looks cheaper from about 2,000 instances, but only against a CPU path
that lanes made slower than the serial one.

## Conclusion

**`GPU_CULLING_MIN_INSTANCES` stays 16,384.** With the cull serial (the
web's single-threaded wasm, native with lanes off), the crossover is at
about 16,000 instances on the occluded street and between 16,000 and 32,000
from the air, which is where the constant already is: below it the GPU path
costs up to twice the CPU path's `render` time, above 32,000 it saves a
quarter to a third. The review fixes did lower the GPU path's fixed cost
(about 1.2 to 1.5 ms here at load 20 to 30, against the 0.4 to 1 ms
penalty "on scenes of tens" reported before on a machine at load 25 to 200),
but not enough to move the crossover by the data.

The lanes-on numbers argue for a lower threshold natively (GPU cheaper from
about 2,000 to 4,000), but what they measure is the CPU cull's lanes split
costing more than it saves under load, not the GPU path getting cheaper;
lowering the threshold to hide that would be the wrong fix, and the web's
single-threaded wasm (the shipping case on WebGPU) never splits. Left
open, outside this measurement: the cull's lanes grain (2,048 bounds is
too fine on a loaded machine, as the lanes round already found for the
reference scene), an idle-machine run (every figure here is at load 20 to
33), and the same sweep in Chrome on WebGPU, where the earlier round found
the GPU path relatively cheaper (35 % at 40,000) because the wasm's CPU
cull costs more than native code's.

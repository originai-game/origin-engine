# Lantern Vale's frame time

A 3D top-down vale on a streamed terrain: 74 entities, three skinned,
45 instances in 11 batches, drop-shadowed sprites over the terrain, 25
render-graph passes, 69 draws, under its look (the anime-cel preset's
toon light, ink outlines, TAA, bloom, auto exposure, fog, a vignette).
The method, the builds and the load are in README.md.

## Before and after, at the lightest load of the round

After the wave-end VM stopped (from 23:10), on main at 48133d07 merged
(the before addon main plus 2b61462a): the play run as fast as it goes
(`--headless`), six interleaved rounds of before and after on each addon, load average 6.7 to 42.8 (median 12.2).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 0.90 | 1.88 | 3.06 | 2.14 | 7.17 | 0fb19ee7ce2ba3c7 |
| release | after | 0.71 | 1.78 | 3.64 | 3.71 | 7.79 | 0fb19ee7ce2ba3c7 |
| dev | before | 2.77 | 4.55 | 15.89 | 2.49 | 3.29 | 0fb19ee7ce2ba3c7 |
| dev | after | 2.13 | 4.98 | 12.57 | 2.27 | 3.71 | 0fb19ee7ce2ba3c7 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `Ui` | 0.293 | 0.205 | 0.873 | 0.492 |
| `og.physics` | 0.140 | 0.135 | 0.202 | 0.224 |
| `Slimes` | 0.129 | 0.101 | 0.460 | 0.376 |
| `Hero` | 0.107 | 0.092 | 0.288 | 0.250 |
| `og.ui.end` | 0.085 | 0.085 | 0.441 | 0.467 |
| `Lanterns` | 0.071 | 0.068 | 0.161 | 0.143 |
| `View` | 0.034 | 0.031 | 0.083 | 0.083 |
| `og.physics.queries` | 0.029 | 0.030 | 0.070 | 0.071 |
| `og.orbitCamera` | 0.018 | 0.019 | 0.028 | 0.031 |
| `og.ui.begin` | 0.016 | 0.015 | 0.056 | 0.056 |

Unpaced, the render column is the GPU's (below), on a GPU the
coordinator's gate was also using: its rounds ran from 1.3 to 6.7 ms
median for the same command stream. Paced, it is the engine thread's:

The same, paced at 60 Hz (`og run` without `--headless`: a frame each
16.7 ms of wall clock, so the GPU is never behind and the render column
is the engine thread's own; a paced run can end a tick past 600, so its
hash is not compared), three interleaved rounds of before and after on each addon, load average 8.0 to 27.7 (median 12.8).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 0.86 | 1.42 | 2.36 | 1.22 | 1.48 | 0fb19ee7ce2ba3c7, 2244eea9d6a667a4 |
| release | after | 0.74 | 1.29 | 2.48 | 1.18 | 1.60 | 0fb19ee7ce2ba3c7 |
| dev | before | 2.66 | 3.90 | 11.05 | 2.39 | 2.97 | 0fb19ee7ce2ba3c7 |
| dev | after | 1.93 | 2.78 | 9.89 | 2.07 | 2.40 | 0fb19ee7ce2ba3c7 |

## Before and after, at a load average of 20 to 25

The first measurement, on main at c955082d merged, while the wave-end
VM ran: three interleaved rounds of before and after on each addon, load average 21.7 to 25.2 (median 23.2).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 1.52 | 3.74 | 6.84 | 3.35 | 13.97 | 0fb19ee7ce2ba3c7 |
| release | after | 1.28 | 3.08 | 10.03 | 5.53 | 16.41 | 0fb19ee7ce2ba3c7 |
| dev | before | 5.67 | 14.85 | 49.05 | 4.78 | 9.53 | 0fb19ee7ce2ba3c7 |
| dev | after | 4.12 | 9.33 | 22.29 | 3.88 | 8.43 | 0fb19ee7ce2ba3c7 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `Ui` | 0.521 | 0.337 | 1.848 | 0.902 |
| `og.physics` | 0.210 | 0.238 | 0.400 | 0.355 |
| `Slimes` | 0.235 | 0.183 | 1.002 | 0.761 |
| `og.ui.end` | 0.147 | 0.159 | 0.889 | 0.781 |
| `Hero` | 0.179 | 0.152 | 0.706 | 0.457 |
| `Lanterns` | 0.124 | 0.120 | 0.353 | 0.267 |
| `View` | 0.061 | 0.060 | 0.191 | 0.151 |
| `og.physics.queries` | 0.045 | 0.051 | 0.142 | 0.130 |
| `og.orbitCamera` | 0.032 | 0.033 | 0.072 | 0.060 |
| `og.ui.begin` | 0.024 | 0.030 | 0.097 | 0.091 |

## Where the frame goes

macOS `sample` of the engine thread for 8 s of a longer play run, at a
load average of 7.5 to 10.5; each row the share
of `og_engine_frame`'s samples whose stack holds it, nested rows within
their parent):

| Where | lantern-vale-rel-before | lantern-vale-rel-after | lantern-vale-dev-before | lantern-vale-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 15.2 % | 10.2 % | 49.1 % | 43.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 10.1 % | 5.3 % | 34.3 % | 27.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 0.9 % | 0.1 % | 3.0 % | 0.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 2.0 % | 0.8 % | 10.2 % | 8.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 0.4 % | 0.1 % | 4.6 % | 0.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 2.5 % | 2.6 % | 3.5 % | 4.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 0.0 % | 0.1 % | 0.1 % | 0.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 0.2 % | 0.1 % | 1.3 % | 1.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 0.4 % | 0.2 % | 1.4 % | 1.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 0.7 % | 0.5 % | 2.9 % | 3.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 2.0 % | 0.0 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 84.3 % | 89.2 % | 47.9 % | 53.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 5.2 % | 4.4 % | 19.4 % | 17.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 0.7 % | 0.3 % | 2.2 % | 0.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 6.6 % | 3.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 0.9 % | 0.7 % | 1.8 % | 2.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 0.6 % | 0.5 % | 0.6 % | 1.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 79.2 % | 84.9 % | 28.4 % | 35.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 70.2 % | 72.3 % | 13.2 % | 16.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 2.2 % | 2.3 % | 3.1 % | 4.0 % |

Samples in og_engine_frame: 3370 of 3399, 3019 of 3044, 4257 of 4544, 3258 of 3488

## The render is the GPU's

On release, 57-66 % of the engine thread's samples are inside
`CommandEncoder::finish`, waiting in Metal's `commandBuffer` for a
semaphore: wgpu records each render pass in two command buffers of its
own (the pass, and the barriers before it), and when the GPU is behind,
making the next one blocks until an earlier one retires. The render
milliseconds of a headless native run are then the GPU's time for the
frame, here a GPU shared with an iOS simulator's Metal host, which is
why before and after (identical command streams) differ by more than
their spread.

The look's features, each turned off at 0.3 s of the same play run
(`set:og.RenderSettings.bloom=false` and so on; release, after, three
interleaved rounds, load average 15 to 23; the hash moves, as the frames
do):

| Off | render median | render p95 | the rounds' medians |
|---|---|---|---|
| nothing (the look as shipped) | 6.11 | 8.25 | 7.46, 5.96, 6.11 |
| bloom | 3.51 | 6.03 | 1.97, 3.62, 3.51 |
| auto exposure | 5.11 | 7.62 | 3.78, 5.11, 5.27 |
| ink outlines | 5.77 | 8.34 | 3.66, 5.84, 5.77 |
| fog | 6.20 | 8.08 | 6.20, 6.42, 6.13 |
| vignette | 6.09 | 8.08 | 6.14, 6.09, 5.75 |
| all five | 1.60 | 4.33 | 1.60, 2.02, 1.52 |

Bloom (a prefilter and five levels down and up: eleven passes, 22
command buffers) is about half of the frame's GPU time on this GPU;
auto exposure (its metering chain) the next. Neither is a redundant pass:
both are on in the look. The game's README has the advice.

## What was the engine's, and what changed

The shares are of the low-load profile above (release, dev).

- **Drop shadows over the terrain sampled normals they threw away**
  (ground_drop_shadows, 6.6 % of a dev frame): b2d8ef7d samples the
  heights alone; 3.2 %.
- **og.i18n.t** (0.9 %, 3.0 %): 381e3d59, 0.1 %, 0.9 %.
- **Record calls** (og.world.get and set, 2.0 %, 10.2 %): f3ad73cb,
  0.8 %, 8.5 %.
- **og.ui.frameTheme** (0.4 %, 4.6 %): 3b2d1c21, 0.1 %, 0.1 %.
- **Match counts** (2.0 % dev): 0b87fc55, 0.0 %.

## What is the game's

Step: `Ui` (0.21 / 0.49 ms at the low load), og.physics (0.14 / 0.22),
`Slimes` (0.10 / 0.38: each slime's Slime and Transform read with
`world.get` and written with `world.set` every tick), `Hero`, og.ui.end.
Render: the look (above). examples/lantern-vale/README.md has the
advice.

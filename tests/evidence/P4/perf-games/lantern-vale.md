# Lantern Vale's frame time

A 3D top-down vale on a streamed terrain: 74 entities, three skinned,
45 instances in 11 batches, drop-shadowed sprites over the terrain, 25
render-graph passes, 69 draws, under its look (the anime-cel preset's
toon light, ink outlines, TAA, bloom, auto exposure, fog, a vignette).
The method, the builds and the load are in README.md.

## Before and after

Play runs: 3 interleaved rounds of before and after on each addon, load average 21.7 to 25.2 (median 23.2).

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

Where the frame goes (macOS `sample` of the engine thread over 8 s of a longer play run, the share of `og_engine_frame`'s samples):

| Where | lantern-vale-rel-before | lantern-vale-rel-after | lantern-vale-dev-before | lantern-vale-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 23.7 % | 16.0 % | 51.5 % | 46.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 16.4 % | 10.4 % | 36.2 % | 31.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 1.7 % | 0.1 % | 3.4 % | 0.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 2.9 % | 0.9 % | 9.9 % | 7.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 1.1 % | 0.1 % | 4.4 % | 0.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 2.7 % | 2.1 % | 3.2 % | 3.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 0.0 % | 0.1 % | 0.1 % | 0.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 0.5 % | 0.2 % | 1.6 % | 1.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 0.2 % | 0.1 % | 1.3 % | 1.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 1.0 % | 1.3 % | 3.0 % | 4.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 2.3 % | 0.1 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 75.1 % | 83.0 % | 44.8 % | 48.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 4.3 % | 2.7 % | 16.7 % | 15.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 0.5 % | 0.1 % | 2.3 % | 1.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 4.3 % | 2.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 0.5 % | 0.6 % | 1.4 % | 1.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 0.4 % | 0.2 % | 0.9 % | 0.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 70.8 % | 80.3 % | 28.1 % | 33.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 57.2 % | 65.7 % | 12.3 % | 14.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 4.8 % | 6.2 % | 3.4 % | 4.3 % |

Samples in og_engine_frame: 4109 of 4183, 4868 of 4924, 4627 of 5074, 4556 of 5094

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

- **Drop shadows over the terrain sampled normals they threw away**
  (ground_drop_shadows, 4.3-5.6 % of a dev frame): b2d8ef7d samples the
  heights alone; 2.0-2.6 %.
- **og.i18n.t** (1.7 % release, 3.4 % dev): 381e3d59, 0.1 % and 0.8 %.
- **Record calls** (og.world.get and set, 2.9 % release, 9.9 % dev):
  f3ad73cb, 0.9 % and 7.9 %.
- **og.ui.frameTheme** (1.1 % release, 4.4 % dev): 3b2d1c21, 0.1-0.3 %.
- **Match counts** (2.3 % dev): 0b87fc55, 0.1 %.

## What is the game's

Step: `Ui` (0.34 / 0.90 ms), og.physics, `Slimes` (0.18 / 0.76: each
slime's Slime and Transform read with `world.get` and written with
`world.set` every tick), og.ui.end, `Hero`. Render: the look (above).

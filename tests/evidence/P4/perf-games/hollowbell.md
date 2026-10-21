# Hollowbell's frame time

A 2D platformer on a 96 x 48 tile map: 206 sprites a frame (33 of them
the overlay's), 1,572 tile quads, 24 2D lights, 18 particle systems, 13
render-graph passes, 33 draws, under the 16-bit pixel look (a 240-line
grid, a 32-colour palette through a Bayer dither, the 2D glow). The
method, the builds and the load are in README.md.

## Before and after, at the lightest load of the round

After the wave-end VM stopped (from 23:10), on main at 48133d07 merged
(the before addon main plus 2b61462a): the play run as fast as it goes
(`--headless`), six interleaved rounds of before and after on each addon, load average 6.7 to 41.0 (median 11.1).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 0.66 | 1.36 | 2.94 | 0.70 | 1.15 | e24062103e2b2391 |
| release | after | 0.53 | 1.09 | 2.45 | 0.60 | 1.42 | e24062103e2b2391 |
| dev | before | 3.06 | 4.58 | 8.52 | 2.92 | 3.61 | e24062103e2b2391 |
| dev | after | 2.06 | 3.70 | 9.78 | 1.90 | 2.60 | e24062103e2b2391 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `Ui` | 0.284 | 0.132 | 1.174 | 0.458 |
| `Control` | 0.069 | 0.069 | 0.224 | 0.208 |
| `og.physics` | 0.053 | 0.059 | 0.169 | 0.165 |
| `Enemies` | 0.034 | 0.037 | 0.108 | 0.106 |
| `Pickups` | 0.036 | 0.037 | 0.118 | 0.116 |
| `og.ui.end` | 0.033 | 0.034 | 0.240 | 0.237 |
| `og.follow2D` | 0.024 | 0.026 | 0.074 | 0.070 |
| `Flickers` | 0.019 | 0.022 | 0.076 | 0.075 |
| `Lanterns` | 0.020 | 0.021 | 0.075 | 0.072 |
| `og.physics.queries` | 0.018 | 0.019 | 0.076 | 0.073 |

The same, paced at 60 Hz (`og run` without `--headless`: a frame each
16.7 ms of wall clock, so the GPU is never behind and the render column
is the engine thread's own; a paced run can end a tick past 600, so its
hash is not compared), three interleaved rounds of before and after on each addon, load average 8.1 to 24.6 (median 12.1).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 1.00 | 1.59 | 3.30 | 0.90 | 1.36 | a6201b21f6374ae8, e24062103e2b2391 |
| release | after | 0.79 | 1.21 | 5.17 | 0.74 | 0.89 | e24062103e2b2391 |
| dev | before | 3.19 | 4.08 | 8.28 | 2.95 | 3.34 | e24062103e2b2391 |
| dev | after | 2.21 | 3.04 | 6.91 | 1.94 | 2.19 | e24062103e2b2391 |

## Before and after, at a load average of 20 to 25

The first measurement, on main at c955082d merged, while the wave-end
VM ran: three interleaved rounds of before and after on each addon, load average 20.8 to 24.8 (median 23.1).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 1.27 | 3.71 | 11.95 | 1.17 | 3.21 | e24062103e2b2391 |
| release | after | 1.09 | 2.85 | 8.49 | 1.14 | 2.56 | e24062103e2b2391 |
| dev | before | 4.95 | 9.86 | 17.57 | 4.45 | 7.75 | e24062103e2b2391 |
| dev | after | 3.80 | 9.62 | 18.35 | 3.12 | 6.61 | e24062103e2b2391 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `Ui` | 0.551 | 0.277 | 1.907 | 0.908 |
| `Control` | 0.157 | 0.136 | 0.378 | 0.425 |
| `og.physics` | 0.113 | 0.114 | 0.262 | 0.279 |
| `Enemies` | 0.074 | 0.075 | 0.191 | 0.203 |
| `Pickups` | 0.077 | 0.074 | 0.207 | 0.247 |
| `og.ui.end` | 0.068 | 0.068 | 0.371 | 0.457 |
| `og.follow2D` | 0.072 | 0.054 | 0.135 | 0.151 |
| `Lanterns` | 0.047 | 0.042 | 0.143 | 0.155 |
| `Flickers` | 0.041 | 0.038 | 0.144 | 0.152 |
| `og.physics.queries` | 0.039 | 0.037 | 0.126 | 0.141 |

## Where the frame goes

macOS `sample` of the engine thread for 8 s of a longer play run, at a
load average of 6.5 to 9.0; each row the share
of `og_engine_frame`'s samples whose stack holds it, nested rows within
their parent):

| Where | hollowbell-rel-before | hollowbell-rel-after | hollowbell-dev-before | hollowbell-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 47.6 % | 28.9 % | 49.8 % | 48.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 34.9 % | 19.6 % | 35.0 % | 33.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 9.1 % | 0.2 % | 8.8 % | 0.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 2.2 % | 0.2 % | 3.6 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 1.7 % | 2.2 % | 2.9 % | 3.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 3.7 % | 3.0 % | 2.9 % | 3.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 0.1 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 0.0 % | 0.1 % | 0.2 % | 0.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 0.6 % | 0.4 % | 1.3 % | 1.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 1.2 % | 1.0 % | 2.1 % | 2.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 3.1 % | 0.0 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 51.8 % | 70.3 % | 48.5 % | 49.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 22.6 % | 9.0 % | 32.9 % | 26.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 16.6 % | 5.2 % | 25.8 % | 15.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 2.5 % | 2.0 % | 3.4 % | 6.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 0.7 % | 0.6 % | 0.9 % | 1.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 29.2 % | 61.3 % | 15.5 % | 23.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 13.5 % | 44.8 % | 3.9 % | 6.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 4.2 % | 5.5 % | 1.2 % | 2.4 % |

Samples in og_engine_frame: 4028 of 4129, 3249 of 3329, 4428 of 4800, 4290 of 4674

## What was the engine's, and what changed

The shares are of the low-load profile above (release, dev).

- **og.i18n.t copied every string table on each call** (9.1 %, 8.8 %):
  the HUD translates a few dozen strings a tick. 381e3d59; 0.2 %, 0.4 %.
  The game's `Ui` system fell to half on both addons.
- **Every sprite, flipbook and tile layer row was decoded to JSON each
  frame** (feel's extraction, 16.6 %, 25.8 %). 2c35ae03 reads them in
  place; 5.2 %, 15.9 %, what is left being the field lookups (README.md,
  owed).
- **og.ui.frameTheme rebuilt the theme's JSON each tick** (2.2 %, 3.6 %):
  3b2d1c21, 0.2 %, 0.0 %.
- **Each system's match counts were counted again each tick** (3.1 % of
  a dev frame): 0b87fc55, 0.0 %.

## What is the game's

The `Ui` system (the HUD, the title and the screens built each tick
through og.ui, og.i18n and the kit) is the slowest system on both
addons (0.13 ms release, 0.46 dev at the low load), then `Control` (the
hero's input and moves, 0.07 / 0.21) and og.physics (0.06 / 0.17).
Nothing is near the 8 ms line on release (step p95 1.09, render p95
1.42) nor the 16 ms one on dev (3.70). examples/hollowbell/README.md's
Frame time paragraph has the advice.

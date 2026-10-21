# The Kestrel Stair's frame time

A 3D climb: 93 entities, eleven skinned instances, four cloth banners
(8 x 14 points, 10 iterations) and the courier's cape (7 x 12) against
the level's triangle mesh, a rope, seven particle systems, 20
render-graph passes, 74 draws. The method, the builds and the load are in
README.md.

## Before and after, at the lightest load of the round

After the wave-end VM stopped (from 23:10), on main at 48133d07 merged
(the before addon main plus 2b61462a): the play run as fast as it goes
(`--headless`), six interleaved rounds of before and after on each addon, load average 6.8 to 39.2 (median 11.0).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 1.98 | 2.90 | 5.13 | 1.21 | 2.63 | ec228f8a48073301 |
| release | after | 1.28 | 2.31 | 7.23 | 1.50 | 4.40 | ec228f8a48073301 |
| dev | before | 5.29 | 8.85 | 29.19 | 2.24 | 3.69 | ec228f8a48073301 |
| dev | after | 2.98 | 4.00 | 8.68 | 1.98 | 2.50 | ec228f8a48073301 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `og.physics` | 0.983 | 0.494 | 1.188 | 0.491 |
| `Ui` | 0.438 | 0.251 | 1.548 | 0.497 |
| `Hero` | 0.112 | 0.121 | 0.371 | 0.262 |
| `Foes` | 0.115 | 0.098 | 0.635 | 0.360 |
| `og.audio` | 0.052 | 0.058 | 0.247 | 0.213 |
| `Fx` | 0.058 | 0.055 | 0.298 | 0.198 |
| `og.ui.end` | 0.041 | 0.048 | 0.212 | 0.184 |
| `View` | 0.040 | 0.045 | 0.114 | 0.093 |
| `Pickups` | 0.034 | 0.039 | 0.110 | 0.086 |
| `og.ai` | 0.035 | 0.039 | 0.161 | 0.140 |

Unpaced, the render column is the GPU's whenever the engine thread
outruns it: a faster step submits the next frame sooner and waits longer
in the encoder's finish for the GPU (step plus render, medians: before
3.19 ms, after 2.78), which is why render rose there with nothing in it
changed but the extraction. Paced, it does not:

The same, paced at 60 Hz (`og run` without `--headless`: a frame each
16.7 ms of wall clock, so the GPU is never behind and the render column
is the engine thread's own; a paced run can end a tick past 600, so its
hash is not compared), three interleaved rounds of before and after on each addon, load average 8.3 to 31.2 (median 12.3).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 2.00 | 4.02 | 20.19 | 1.04 | 1.71 | ec228f8a48073301 |
| release | after | 1.14 | 1.68 | 6.52 | 0.97 | 1.26 | ec228f8a48073301 |
| dev | before | 4.96 | 6.89 | 13.46 | 2.14 | 2.62 | ec228f8a48073301 |
| dev | after | 2.98 | 3.79 | 8.94 | 1.95 | 2.34 | ec228f8a48073301 |

## Before and after, at a load average of 20 to 25

The first measurement, on main at c955082d merged, while the wave-end
VM ran: three interleaved rounds of before and after on each addon, load average 20.8 to 24.3 (median 23.5).

| Addon | | step median | step p95 | step max | render median | render p95 | world hash |
|---|---|---|---|---|---|---|---|
| release | before | 6.59 | 17.20 | 28.40 | 2.02 | 3.91 | ec228f8a48073301 |
| release | after | 3.53 | 17.66 | 41.18 | 1.87 | 3.23 | ec228f8a48073301 |
| dev | before | 14.42 | 33.48 | 47.02 | 3.95 | 7.19 | ec228f8a48073301 |
| dev | after | 8.46 | 23.33 | 46.89 | 3.94 | 6.63 | ec228f8a48073301 |

Systems by name (the median over the rounds of each run's mean ms a tick; the ten slowest after, release):

| System | release before | release after | dev before | dev after |
|---|---|---|---|---|
| `og.physics` | 5.443 | 4.186 | 7.959 | 5.222 |
| `Ui` | 0.916 | 0.383 | 2.857 | 1.063 |
| `Hero` | 0.232 | 0.193 | 0.776 | 0.565 |
| `Foes` | 0.265 | 0.158 | 1.171 | 0.775 |
| `Fx` | 0.114 | 0.091 | 0.591 | 0.393 |
| `og.audio` | 0.086 | 0.088 | 0.473 | 0.499 |
| `View` | 0.079 | 0.073 | 0.237 | 0.230 |
| `og.ui.end` | 0.071 | 0.072 | 0.378 | 0.352 |
| `Pickups` | 0.083 | 0.066 | 0.275 | 0.202 |
| `og.ai` | 0.064 | 0.060 | 0.294 | 0.297 |

## Where the frame goes

macOS `sample` of the engine thread for 8 s of a longer play run, at a
load average of 7.3 to 8.1; each row the share
of `og_engine_frame`'s samples whose stack holds it, nested rows within
their parent):

| Where | kestrel-stair-rel-before | kestrel-stair-rel-after | kestrel-stair-dev-before | kestrel-stair-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 66.9 % | 52.7 % | 69.0 % | 59.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 26.4 % | 25.3 % | 39.9 % | 32.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 6.9 % | 0.2 % | 9.1 % | 0.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 3.7 % | 4.0 % | 10.3 % | 9.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 1.0 % | 0.1 % | 3.1 % | 0.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 0.0 % | 0.0 % | 0.0 % | 0.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 31.5 % | 16.0 % | 11.7 % | 8.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 28.9 % | 13.0 % | 9.1 % | 5.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 1.1 % | 1.5 % | 2.7 % | 2.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 1.3 % | 1.6 % | 3.3 % | 3.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 1.1 % | 1.6 % | 2.3 % | 2.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 3.0 % | 0.7 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 31.9 % | 46.4 % | 29.0 % | 37.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 5.1 % | 5.5 % | 9.4 % | 10.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 0.4 % | 0.0 % | 0.5 % | 0.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 0.6 % | 1.3 % | 1.5 % | 1.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 1.0 % | 1.4 % | 2.5 % | 2.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 26.8 % | 40.9 % | 19.6 % | 27.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 13.9 % | 22.7 % | 6.5 % | 8.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 5.5 % | 4.9 % | 2.2 % | 3.2 % |

Samples in og_engine_frame: 2653 of 2793, 2372 of 2511, 3502 of 3980, 3193 of 3770

## With the lanes off

The same play run with `OG_LANES=off` (every pass on the engine thread;
the world hash is the same, ec228f8a48073301), three interleaved rounds,
load average 20.1 to 22.5:

| Addon | | step median | step p95 | render median | render p95 | runs | hash |
|---|---|---|---|---|---|---|---|
| release | before | 5.39 | 10.48 | 1.66 | 4.45 | 3 | ec228f8a48073301 |
| release | after | 2.09 | 4.45 | 1.69 | 4.39 | 3 | ec228f8a48073301 |
| dev | before | 10.29 | 17.64 | 3.31 | 7.12 | 3 | ec228f8a48073301 |
| dev | after | 6.58 | 12.52 | 3.46 | 8.44 | 3 | ec228f8a48073301 |

og.physics a tick (the median of the rounds' means): release 3.665 →
0.932 ms, dev 3.281 → 1.253 ms.

## The cloth, and the lanes

Before, a cloth point's collision projected it onto the level's mesh
with parry's unbounded search for the nearest triangle, once per point
per iteration, to leave alone every point farther than its radius (all
of them but those touching): 89 % of og.physics. 465ab16d bounds the
search at the radius (the same triangle when one is within it, so the
same arithmetic and hash). On the lanes' threads (sampled with the
engine thread for 8 s, eight lane threads), the cloth's own work
(`ClothState::step`) fell from 1,989 samples to 557 at the low load
(1,252 to 338 at 20 to 25), the rest of those threads' time being
rayon's sleep and spin.

What is left on a loaded machine is the wait: the engine thread hands
the five cloths to the pool (their work, 5,320 point-passes, is over two
of the eight-thread grain's 2,000) and blocks until threads the OS has
not scheduled run them. Serially og.physics is 0.93 ms; on the pool it
was 2.9 to 4.2 ms at a load of 20 to 25 and its p95 carries the step's;
at 7 to 16 the pool pays again (0.49 ms).
The grains were measured on an idle machine; README.md owes the lanes'
behaviour under load.

## What was the engine's, and what changed

The shares are of the low-load profile above (release, dev).

- **The cloth's unbounded mesh projections** (the soft phase 28.9 %,
  9.1 %): 465ab16d, 13.0 %, 5.4 %; og.physics a tick 0.98 → 0.49 ms
  release, 1.19 → 0.49 dev (the six unpaced rounds' medians).
- **og.i18n.t copied every string table** (6.9 %, 9.1 %): 381e3d59,
  0.2 %, 0.5 %. `Ui` 0.44 → 0.25 ms release, 1.55 → 0.50 dev.
- **Record calls copied their layout and answer** (og.world.get and set,
  3.7 %, 10.3 %): f3ad73cb; 4.0 %, 9.1 % (the rest is the JSON both ways,
  the call's cost model, and the game's per-entity calls).
- **og.ui.frameTheme** (1.0 %, 3.1 %): 3b2d1c21, 0.1 %, 0.2 %.
- **Match counts each tick** (3.0 % dev): 0b87fc55, 0.7 %.

## What is the game's

After the cloth, `Ui` (0.25 / 0.50 ms at the low load), `Hero` (0.12 /
0.26) and `Foes` (0.10 / 0.36) lead. The game's per-entity `world.get`
and `world.set` (the hero's and the foes' Transform,
CharacterController, AnimationPlayer and Foe, each a JSON round trip)
are 4 % of a release frame and 9 % of a dev one. examples/kestrel-stair/README.md
has the advice (og-game §14).

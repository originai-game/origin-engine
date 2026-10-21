# The Kestrel Stair's frame time

A 3D climb: 93 entities, eleven skinned instances, four cloth banners
(8 x 14 points, 10 iterations) and the courier's cape (7 x 12) against
the level's triangle mesh, a rope, seven particle systems, 20
render-graph passes, 74 draws. The method, the builds and the load are in
README.md.

## Before and after

Play runs: 3 interleaved rounds of before and after on each addon, load average 20.8 to 24.3 (median 23.5).

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

Where the frame goes (macOS `sample` of the engine thread over 8 s of a longer play run, the share of `og_engine_frame`'s samples):

| Where | kestrel-stair-rel-before | kestrel-stair-rel-after | kestrel-stair-dev-before | kestrel-stair-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 70.6 % | 74.7 % | 70.6 % | 67.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 24.8 % | 15.8 % | 34.3 % | 27.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 7.5 % | 0.2 % | 7.2 % | 0.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 2.6 % | 2.1 % | 8.0 % | 7.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 0.9 % | 0.2 % | 2.6 % | 0.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 0.0 % | 0.1 % | 0.0 % | 0.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 37.5 % | 50.7 % | 23.1 % | 24.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 36.2 % | 48.1 % | 21.2 % | 22.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 0.4 % | 1.0 % | 1.9 % | 2.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 1.3 % | 0.7 % | 2.4 % | 2.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 1.9 % | 1.3 % | 1.4 % | 2.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 2.4 % | 1.3 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 28.6 % | 24.7 % | 27.6 % | 30.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 2.7 % | 2.4 % | 8.4 % | 10.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 0.4 % | 0.2 % | 0.5 % | 0.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 0.4 % | 0.5 % | 1.3 % | 1.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 0.2 % | 0.7 % | 1.8 % | 2.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 25.9 % | 22.3 % | 19.1 % | 20.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 8.2 % | 12.0 % | 6.3 % | 7.4 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 12.6 % | 4.1 % | 2.3 % | 1.6 % |

Samples in og_engine_frame: 3590 of 3734, 3608 of 3723, 3386 of 3806, 2935 of 3383

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
engine thread, 8 s, eight lane threads), the cloth's own work fell from
1,252 samples to 338 (`ClothState::step`), the rest of those threads'
time being rayon's spin (13,287 → 21,294) and sleep (15,247 → 8,075).

What is left on a loaded machine is the wait: the engine thread hands
the five cloths to the pool (their work, 5,320 point-passes, is over two
of the eight-thread grain's 2,000) and blocks until threads the OS has
not scheduled run them. Serially og.physics is 0.93 ms; on the pool it
was 2.9 to 4.2 ms at a load of 20 to 25 and its p95 carries the step's.
The grains were measured on an idle machine; README.md owes the lanes'
behaviour under load.

## What was the engine's, and what changed

- **The cloth's unbounded mesh projections** (above): 465ab16d.
- **og.i18n.t copied every string table** (7.2-7.5 % of the frame):
  381e3d59, 0.2-0.4 %. `Ui` 0.92 → 0.38 ms release, 2.86 → 1.06 dev.
- **Record calls copied their layout and answer** (og.world.get and set,
  8.0 % of a dev frame): f3ad73cb; 7.3 % (the rest is the JSON both ways,
  the call's cost model).
- **og.ui.frameTheme** (2.6 % dev): 3b2d1c21, 0.1 %.
- **Match counts each tick** (2.4 % dev): 0b87fc55.

## What is the game's

After the cloth, `Ui` (0.38 / 1.06 ms) and `Foes` (0.16 / 0.78) lead.
The game's per-entity `world.get` and `world.set` (the hero's and the
foes' Transform, CharacterController, AnimationPlayer and Foe, each a
JSON round trip) are 7 % of a dev frame. The game's README has the
advice (og-game §14).

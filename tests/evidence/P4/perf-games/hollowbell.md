# Hollowbell's frame time

A 2D platformer on a 96 x 48 tile map: 206 sprites a frame (33 of them
the overlay's), 1,572 tile quads, 24 2D lights, 18 particle systems, 13
render-graph passes, 33 draws, under the 16-bit pixel look (a 240-line
grid, a 32-colour palette through a Bayer dither, the 2D glow). The
method, the builds and the load are in README.md.

## Before and after

Play runs: 3 interleaved rounds of before and after on each addon, load average 20.8 to 24.8 (median 23.1).

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

Where the frame goes (macOS `sample` of the engine thread over 8 s of a longer play run, the share of `og_engine_frame`'s samples):

| Where | hollowbell-rel-before | hollowbell-rel-after | hollowbell-dev-before | hollowbell-dev-after |
|---|---|---|---|---|
| frame (og_engine_frame) | 100.0 % | 100.0 % | 100.0 % | 100.0 % |
| &nbsp;&nbsp;step: World::step | 46.5 % | 48.8 % | 50.4 % | 52.6 % |
| &nbsp;&nbsp;&nbsp;&nbsp;game systems (TypeScript, with their engine calls) | 33.2 % | 32.3 % | 36.9 % | 37.8 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.i18n.t | 7.1 % | 0.4 % | 7.7 % | 0.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.world.get / set | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;og.ui.frameTheme | 2.0 % | 0.3 % | 3.8 % | 0.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;column views | 2.8 % | 3.2 % | 3.2 % | 4.3 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.physics | 3.3 % | 5.2 % | 2.4 % | 3.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;cloth and ropes (soft) | 0.0 % | 0.1 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.anim | 0.0 % | 0.1 % | 0.2 % | 0.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.audio | 0.4 % | 0.8 % | 1.2 % | 1.9 % |
| &nbsp;&nbsp;&nbsp;&nbsp;og.ui.end (layout) | 1.6 % | 1.7 % | 1.5 % | 2.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;match counts | 0.0 % | 0.0 % | 3.3 % | 0.3 % |
| &nbsp;&nbsp;render: extract and Renderer::render | 52.0 % | 49.8 % | 47.1 % | 44.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;extract | 15.8 % | 12.4 % | 31.7 % | 23.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;sprites, flipbooks, tiles (feel) | 11.4 % | 7.2 % | 24.8 % | 14.2 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;drop shadows on terrain | 0.0 % | 0.0 % | 0.0 % | 0.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;fx | 1.8 % | 2.8 % | 3.7 % | 5.0 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;atmosphere | 0.6 % | 0.5 % | 0.7 % | 1.1 % |
| &nbsp;&nbsp;&nbsp;&nbsp;Renderer::render | 36.2 % | 37.3 % | 15.4 % | 20.5 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;encoder finish (Metal command buffers: GPU back-pressure) | 11.5 % | 17.3 % | 4.6 % | 5.7 % |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;queue submit | 13.3 % | 5.7 % | 1.5 % | 2.0 % |

Samples in og_engine_frame: 5002 of 5154, 4244 of 4405, 4848 of 5330, 4665 of 5305

## What was the engine's, and what changed

- **og.i18n.t copied every string table on each call** (7.1-7.7 % of the
  frame, release and dev): the HUD translates a few dozen strings a tick.
  381e3d59; now 0.4-0.9 %. The game's `Ui` system went from 0.55 to 0.28
  ms a tick (release) and 1.91 to 0.91 (dev).
- **Every sprite, flipbook and tile layer row was decoded to JSON each
  frame** (feel's extraction, 24.8 % of a dev frame, 11.4 % of a release
  one). 2c35ae03 reads them in place; 14.2 % and 7.2 % now, what is left
  being the field lookups (owed in README.md).
- **og.ui.frameTheme rebuilt the theme's JSON each tick** (3.8 % dev,
  2.0 % release): 3b2d1c21, 0.2-0.3 %.
- **Each system's match counts were counted again each tick** (3.3 % of
  a dev frame): 0b87fc55, 0.3 %.

## What is the game's

The `Ui` system (the HUD, the title and the screens built each tick
through og.ui, og.i18n and the kit) is the slowest system on both
addons (0.28 ms release, 0.91 dev), then `Control` (the hero's input and
moves, 0.14 / 0.43) and og.physics (0.11 / 0.28). Nothing in the frame
is near the 8 ms line on release; on dev, step p95 9.6 ms and render p95
6.6 ms are under the 16 ms one. examples/hollowbell/README.md's Frame time paragraph has the
advice.

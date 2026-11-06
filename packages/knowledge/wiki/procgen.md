<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `procgen` module

Procedural generation: gradient noise (Perlin, simplex, FBM), Poisson-disk points, BSP dungeons and wave function collapse, all deterministic. Each generator takes a `seed`; give it the seed of a fork of og.Rng, `world.resource(og.Rng)`'s value `.fork("dungeon").seed` (a fork needs og.Rng among a system's reads only), so the same world and label give the same output and the main stream does not move. A generator draws only from a stream on that seed, in a fixed order, with arithmetic that is bit-identical on every host: the same seed gives the same map natively and on the web, in a replay and after a restore.

Weighted picks and shuffles are og.Rng's and RngStream's own (`weighted`, `permutation`; a shuffle is a permutation applied to an array). og.tilemap.fill writes a generated grid into a tilemap layer.

- **TypeScript**: functions are the namespace `og.procgen`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module procgen {
  doc "Procedural generation: gradient noise (Perlin, simplex, FBM), Poisson-disk points, BSP dungeons and wave function collapse, all deterministic. Each generator takes a `seed`; give it the seed of a fork of og.Rng, `world.resource(og.Rng)`'s value `.fork(\"dungeon\").seed` (a fork needs og.Rng among a system's reads only), so the same world and label give the same output and the main stream does not move. A generator draws only from a stream on that seed, in a fixed order, with arithmetic that is bit-identical on every host: the same seed gives the same map natively and on the web, in a replay and after a restore."
  doc "Weighted picks and shuffles are og.Rng's and RngStream's own (`weighted`, `permutation`; a shuffle is a permutation applied to an array). og.tilemap.fill writes a generated grid into a tilemap layer."
  # declarations follow, each on its own page
```

<sub>Module `procgen` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`NoiseKind`](procgen/type-NoiseKind.md) | enum |  | Which gradient noise. |
| [`FbmDesc`](procgen/type-FbmDesc.md) | desc struct |  | Fractal noise (FBM): `octaves` layers, each `lacunarity` times the frequency and `gain` times the amplitude of the last, divided by the amplitudes' sum, so the value stays about -1 to 1. |
| [`NoiseGridDesc`](procgen/type-NoiseGridDesc.md) | desc struct |  | A grid of noise samples, row-major from (x, y), `step` apart: a height field, a biome map, a cave mask. |
| [`og.procgen.noise`](procgen/fn-noise.md) | function | `(desc: FbmDesc, x: number, y: number, z: number): number` | Noise at one point, about -1 to 1: a pure function of the desc and the point. |
| [`og.procgen.noiseGrid`](procgen/fn-noiseGrid.md) | function | `(desc?: NoiseGridDesc): Result<readonly number[]>` | Noise over a grid in one call, row-major: `values[j * width + i]` is the sample at (x + i * step, y + j * step). |
| [`PoissonDesc`](procgen/type-PoissonDesc.md) | desc struct |  | Points no closer than `radius` to each other, filling an area evenly with no grid look (Bridson 2007): trees in a forest, loot in a field. |
| [`og.procgen.poissonDisk`](procgen/fn-poissonDisk.md) | function | `(desc?: PoissonDesc): Result<readonly Vector2[]>` | Poisson-disk points in [0, width) × [0, height), in the order they were placed. |
| [`CellRect`](procgen/type-CellRect.md) | struct |  | A rectangle of grid cells: column x, row y (from the top), width and height in cells. |
| [`BspDesc`](procgen/type-BspDesc.md) | desc struct |  | A dungeon by binary space partition: the area is split in two along its longer side, again and again, each leaf gets a room and every split's halves are joined by an L-shaped corridor. |
| [`BspResult`](procgen/type-BspResult.md) | struct |  | What og.procgen.bsp made. |
| [`og.procgen.bsp`](procgen/fn-bsp.md) | function | `(desc?: BspDesc): Result<BspResult>` | A BSP dungeon: rooms that never overlap, every floor cell reachable from every other. |
| [`WfcTile`](procgen/type-WfcTile.md) | struct |  | A tile for og.procgen.wfc: its weight and the socket on each edge. |
| [`WfcCell`](procgen/type-WfcCell.md) | struct |  | A cell fixed before generation starts (an entrance, a river's source). |
| [`WfcDesc`](procgen/type-WfcDesc.md) | desc struct |  | Wave function collapse, the simple tiled model: a grid filled with tiles whose edges all fit. |
| [`WfcResult`](procgen/type-WfcResult.md) | struct |  | What og.procgen.wfc made, row-major from the top. |
| [`og.procgen.wfc`](procgen/fn-wfc.md) | function | `(desc?: WfcDesc): Result<WfcResult>` | A grid of tiles whose edges all fit, by wave function collapse. |

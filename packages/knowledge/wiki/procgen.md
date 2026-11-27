<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `procgen` module

Procedural generation: deterministic noise and FBM, Poisson-disk sampling, BSP dungeons, wave function collapse, eroded heightmaps, plants and Voronoi mesh fracture. Bulk buffers are TypedArrays.

Weighted picks and shuffles are og.Rng's own; og.tilemap.fill writes Int32Array grids into tilemap layers, og.terrain.create turns Float32Array heightmaps into live terrains, and og.render.bufferGeometry turns procedural mesh TypedArrays into GPU/physics meshes.

- **TypeScript**: functions are the namespace `og.procgen`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module](index.md).

```ogt
module procgen {
  doc "Procedural generation: deterministic noise and FBM, Poisson-disk sampling, BSP dungeons, wave function collapse, eroded heightmaps, plants and Voronoi mesh fracture. Bulk buffers are TypedArrays."
  doc "Weighted picks and shuffles are og.Rng's own; og.tilemap.fill writes Int32Array grids into tilemap layers, og.terrain.create turns Float32Array heightmaps into live terrains, and og.render.bufferGeometry turns procedural mesh TypedArrays into GPU/physics meshes."
  # declarations follow, each on its own page
```

<sub>Module `procgen` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`NoiseKind`](procgen/type-NoiseKind.md) | enum |  | Which procedural noise basis or fractal shaping. |
| [`FbmDesc`](procgen/type-FbmDesc.md) | desc struct |  | Fractal noise (FBM): `octaves` layers, each `lacunarity` times the frequency and `gain` times the amplitude of the last, divided by the amplitudes' sum, so the value stays about -1 to 1. |
| [`NoiseGridDesc`](procgen/type-NoiseGridDesc.md) | desc struct |  | A grid of noise samples, row-major from (x, y), `step` apart: a height field, a biome map, a cave mask. |
| [`NoiseGrad`](procgen/type-NoiseGrad.md) | struct |  | Noise value together with its analytical partial derivatives `(dx, dy, dz)` (dz is 0 in 2D). |
| [`og.procgen.noise`](procgen/fn-noise.md) | function | `(desc: FbmDesc, x: number, y: number, z: number): number` | Noise at one point, about -1 to 1: a pure function of the desc and the point. |
| [`og.procgen.noiseGrad`](procgen/fn-noiseGrad.md) | function | `(desc: FbmDesc, x: number, y: number, z: number): NoiseGrad` | Analytical value and gradient of Perlin FBM at one point (exact chain-rule derivatives, no finite-difference jitter). |
| [`og.procgen.curlNoise`](procgen/fn-curlNoise.md) | function | `(desc: FbmDesc, x: number, y: number, z: number): Vector3` | Divergence-free curl vector field from analytical noise gradients: 2D returns `(dN/dy, -dN/dx, 0)`, 3D returns the full 3D curl of three independent potentials. |
| [`og.procgen.worley`](procgen/fn-worley.md) | function | `(seed: number, x: number, y: number, z: number, threeD: boolean, sites?: readonly Vector3[] \| null): Vector2` | Cellular Worley noise distances at a point: `x` is F1 (nearest feature distance) and `y` is F2 (second-nearest feature distance). |
| [`og.procgen.noiseGrid`](procgen/fn-noiseGrid.md) | function | `(desc?: NoiseGridDesc): Result<Float32Array>` | Noise over a grid in one call, row-major as a native Float32Array: `values[j * width + i]` is the sample at (x + i * step, y + j * step). |
| [`PoissonDesc`](procgen/type-PoissonDesc.md) | desc struct |  | Points no closer than `radius` to each other, filling an area evenly with no grid look (Bridson 2007): trees in a forest, loot in a field. |
| [`og.procgen.poissonDisk`](procgen/fn-poissonDisk.md) | function | `(desc?: PoissonDesc): Result<readonly Vector2[]>` | Poisson-disk points in [0, width) × [0, height), in the order they were placed. |
| [`CellRect`](procgen/type-CellRect.md) | struct |  | A rectangle of grid cells: column x, row y (from the top), width and height in cells. |
| [`BspDesc`](procgen/type-BspDesc.md) | desc struct |  | A dungeon by binary space partition: the area is split in two along its longer side, again and again, each leaf gets a room and every split's halves are joined by an L-shaped corridor (plus optional … |
| [`BspResult`](procgen/type-BspResult.md) | struct |  | What og.procgen.bsp made. |
| [`og.procgen.bsp`](procgen/fn-bsp.md) | function | `(desc?: BspDesc): Result<BspResult>` | A BSP dungeon: rooms that never overlap, every floor cell reachable from every other. |
| [`WfcTile`](procgen/type-WfcTile.md) | struct |  | A tile for og.procgen.wfc: its weight and the socket on each edge. |
| [`WfcCell`](procgen/type-WfcCell.md) | struct |  | A cell fixed before generation starts (an entrance, a river's source). |
| [`WfcDesc`](procgen/type-WfcDesc.md) | desc struct |  | Wave function collapse, the simple tiled model with AC-4 constraint propagation and Shannon entropy cell selection: a grid filled with tiles whose edges all fit; a dead end restarts on a fork of the … |
| [`WfcResult`](procgen/type-WfcResult.md) | struct |  | What og.procgen.wfc made, row-major from the top, returned as native TypedArrays over binary wire. |
| [`og.procgen.wfc`](procgen/fn-wfc.md) | function | `(desc?: WfcDesc): Result<WfcResult>` | A grid of tiles whose edges all fit, by wave function collapse. |
| [`HeightmapMask`](procgen/type-HeightmapMask.md) | enum |  | Macro shaping mask applied to generated terrain heights. |
| [`HydraulicErosionDesc`](procgen/type-HydraulicErosionDesc.md) | desc struct |  | Droplet-based hydraulic erosion pass (deterministic seeded drops). |
| [`ThermalErosionDesc`](procgen/type-ThermalErosionDesc.md) | desc struct |  | CFL-stable mass-conserving talus thermal erosion pass. |
| [`RiverCarveDesc`](procgen/type-RiverCarveDesc.md) | desc struct |  | Drainage-accumulation river carving pass. |
| [`MoistureDesc`](procgen/type-MoistureDesc.md) | desc struct |  | Moisture field weights for biome painting. |
| [`BiomeRuleDesc`](procgen/type-BiomeRuleDesc.md) | desc struct |  | One biome painting rule mapping altitude, slope and moisture bands onto a terrain layer index. |
| [`TerrainChannelDesc`](procgen/type-TerrainChannelDesc.md) | desc struct |  | An authored valley or river centre line in terrain-local xyz metres. |
| [`HeightmapDesc`](procgen/type-HeightmapDesc.md) | desc struct |  | Deterministic procedural terrain heightmap, erosion, rivers and biome control map generator (matching `og cook`'s terrain generator bit for bit). |
| [`HeightmapResult`](procgen/type-HeightmapResult.md) | struct |  | Generated procedural heightfield and optional biome control map, ready for og.terrain.create. |
| [`og.procgen.heightmap`](procgen/fn-heightmap.md) | function | `(desc?: HeightmapDesc): Result<HeightmapResult>` | Generate a deterministic eroded terrain heightmap and optional biome control map as native TypedArrays (hand directly to og.terrain.create). |
| [`PlantGrowth`](procgen/type-PlantGrowth.md) | enum |  | Branching architecture of a procedural plant. |
| [`PlantLeafShape`](procgen/type-PlantLeafShape.md) | enum |  | Geometry built for each leaf. |
| [`PlantLevelDesc`](procgen/type-PlantLevelDesc.md) | desc struct |  | Growth parameters for one branch level (level 0 is the trunk). |
| [`PlantLeavesDesc`](procgen/type-PlantLeavesDesc.md) | desc struct |  | Leaf foliage configuration on terminal branches. |
| [`PlantFlowersDesc`](procgen/type-PlantFlowersDesc.md) | desc struct |  | Blossom/flower configuration on terminal branches. |
| [`PlantBranchDesc`](procgen/type-PlantBranchDesc.md) | desc struct |  | One limb of an explicit parent-first plant skeleton. |
| [`PlantBoundsDesc`](procgen/type-PlantBoundsDesc.md) | desc struct |  | Optional centreline growth box in local metres, min < max on every axis. |
| [`PlantDesc`](procgen/type-PlantDesc.md) | desc struct |  | Deterministic procedural tree, shrub or flower mesh specification (og-shape's growth engine). |
| [`PlantMeshPart`](procgen/type-PlantMeshPart.md) | struct |  | One mesh part of a procedural plant (wood or foliage) with vertex and foliage-wind TypedArrays ready for og.render.bufferGeometry. |
| [`PlantMeshResult`](procgen/type-PlantMeshResult.md) | struct |  | Generated procedural plant meshes and statistics. |
| [`og.procgen.plant`](procgen/fn-plant.md) | function | `(desc?: PlantDesc): Result<PlantMeshResult>` | Grow a deterministic procedural tree, shrub or flower into GPU-ready TypedArray mesh buffers (pass directly to og.render.bufferGeometry). |
| [`FractureDesc`](procgen/type-FractureDesc.md) | desc struct |  | Deterministic 3D Voronoi fracture of a closed triangle mesh into solid pieces and Blast-style contact bonds. |
| [`FracturePiece`](procgen/type-FracturePiece.md) | struct |  | One solid fractured shard mesh in the input mesh's coordinate space. |
| [`FractureBond`](procgen/type-FractureBond.md) | struct |  | A shared contact face between two touching fracture pieces. |
| [`FractureResult`](procgen/type-FractureResult.md) | struct |  | Solid pieces and structural contact bonds produced by og.procgen.fracture. |
| [`og.procgen.fracture`](procgen/fn-fracture.md) | function | `(desc?: FractureDesc): Result<FractureResult>` | Fracture a closed mesh deterministically into solid Voronoi shards and contact bonds, returned as TypedArrays. |

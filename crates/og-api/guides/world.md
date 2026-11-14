# Worlds: terrain, streaming, tilemaps, navigation and AI

How to build the ground a game is played on and what moves over it: terrain with its layers, scatter and grass; asset streaming groups and cells for large worlds; tilemaps for 2D; procedural generation; paths; navigation and crowds; and AI behaviours. Everything a tick reads here is deterministic and in snapshots; what only draws (grass, scatter, streamed tiles) never changes the world hash.

The API: `og api show terrain`, `og api show stream`, `og api show tilemap`,
`og api show procgen`, `og api show path`, `og api show nav`,
`og api show ai`.

## 1. Terrain: the file and placing it

A `type terrain` `.ogt` names a 16-bit grey PNG or EXR heightmap, an
optional control map, up to sixteen layered materials with slope and
altitude rules (each pixel blends its four strongest), and an optional
`holes` map whose black or transparent cells are cut out of the surface (a
cave mouth, a pit, a tunnel through a hill). `og cook` cooks it into tiles.

- Layer maps are separate textures for the first four layers, or layers of
  two texture arrays: `layers` and `layerNormals`, with each layer's
  `albedoLayer` and `normalLayer`. Arrays sample two textures instead of up
  to eight and are the only way past four layers; the control map is then
  one image per four layers.
- It draws as a geometry clipmap around the camera (nested levels by
  distance, no cracks) that casts and receives shadows under the fog,
  collides as a height field, and bakes into navmeshes by a navmesh file's
  `terrain` entry.

```ts
const t = og.terrain.load(world, "terrain/valley.ogt").value!;
const ground = world.spawn({ name: "ground" }).value!;
world.insert(ground, og.Terrain, { data: t.data });
world.insert(ground, og.Collider, { shape: 6, mesh: t.geometry });  // SHAPE_HEIGHTFIELD
```

- An og.Terrain draws it placed by its entity's world transform (centred on
  its position, turned and scaled with it). Pass og.terrain.sample the
  same position, quaternion and scale to read the surface's height and
  normal anywhere.
- og.terrain.edit raises, lowers, flattens, smooths or paints at run time;
  og.terrain.setLayer retunes a layer. Edits are world state: a restore
  rebuilds the ground from the cook with the edits replayed, for the
  renderer, physics and sampling alike.
- A stamp's `carve` mode lowers the ground under a mesh so the mesh is the
  surface there (a cliff's face and lip, an arch, an overhang). At the
  edge of what the mesh covers the ground goes down to the underside
  itself, so the ground beside it meets its foot with no trench.
- Terrain is 3D. The 2D profile's publish and lanes builds leave it out
  (RenderCapabilities.world3d is false), and every call answers a refusal
  naming that field.

## 2. Terrain detail and cost

| Setting | Default | Cost |
|---|---|---|
| `lodRing` | the file's, else 64 quads a side (256 for a displacing terrain) | 256 on every level costs about 3 ms more GPU on an M4 Pro; levels grow to 128 on their own where `lodError` needs it |
| `lodError` | 1 px | larger draws fewer vertices |
| `displacementDistance` | 40 m | displaced levels grow finer (down to a 128th of the finest displacing layer's `tile`, a 128th of a sample at most, unless `lodSpacing` is above 0) and denser (256 quads unless `lodRing` says); a cobbled crest took 6.5 ms of GPU a frame on an M4 Pro against 3.8 with 64-quad levels |
| tile budget | 16 MiB per terrain | resident height tiles |

- Displacement is drawn only: within `displacementDistance` (fading over
  the last 40 %) a layer's `displacement` raises and lowers the drawn
  ground by its height map (its albedo's alpha, about the middle), so
  pebbles, cobbles and rock stand out at the silhouette and in shadows; a
  sett's joint and arris are drawn by vertices, not stepped across a cell.
  Collision, og.terrain.sample and picking keep the height field. Each
  corner is evaluated once a frame per pass, so shadow passes pay too.
- A layer's `cellVariation` gives each cell of its pattern (a sett, a flag,
  a tile) its own tone: its normal map's alpha names the cell (any value
  per cell, opaque for none), and at 1 each cell's brightness and warmth
  move by up to 25 % and 20 %, so stones laid in one texture read as stones
  of different quarries.
- Height tiles stream: the cook stores the field as a pyramid of tiles,
  each compressed on its own, and the renderer keeps on the GPU only the
  tiles the clipmap wants, within the budget, decoded off the frame (a
  worker thread natively, DecompressionStream on the web) a few at a time,
  the most needed first. A missing tile draws from the coarser level that
  holds it, then fades in. Collision and sampling read the whole field, so
  the hash is the same streamed or not.
- The probe's `terrain_stream:resident` decodes every wanted tile before
  each frame (for captures); `terrain_stream:stream[:<MiB>]` streams again
  within that budget; `dump_stream`'s `terrain` gives the tiles resident,
  waiting, decoding and cached.

## 3. Scatter and grass

- og.TerrainScatter grows copies of a mesh (grass, flowers, stones) where
  a layer shows, on a jittered grid fixed to the terrain, so each copy
  keeps its place, size and turn as the camera moves; each cell grows one
  with the probability of the layer's weight there. The renderer places a
  chunk of the grid at a time, once, so copies cost about nothing per frame
  while the camera stays. Over the outer `fade` copies shrink to nothing,
  each over its own part of the fade: no dither, so no grain under TAA.
  The copies are not entities and have no colliders.
- GPU placement (on by default) places each chunk by a compute pass when it
  comes into range, by the same rules (the layer's weight, holes,
  noiseThreshold, avoidWater and earlier scatters' footprints) from the
  finest tiles, with nothing per copy on the CPU. It matches the CPU's
  placement but for the last bits; without compute, or with a footprint
  chain too long or too wide, the CPU places them. Use it for dense or far
  fields.
- An impostor (a glTF cooked with `model { impostor true }`) draws copies
  past impostorDistance as an octahedral impostor, cross-dithered over the
  last tenth by blue noise that changes every frame, which TAA resolves to
  a smooth blend. Mesh and impostor bounds must agree in model space; a
  mismatch skips the scatter and reports render.scatterIssue in its entity
  dump and a runtime warning.
- `model` scatters a whole glTF as one copy, in place of geometry and
  material: every part with its own material (a grown plant's wood and its
  leaves, `og api show "models §3"`), placed and swaying together. Cooked
  with an impostor, its copies past impostorDistance draw as it; without
  one its parts must stand at the model's origin, as a grown plant's do.
  A wood is three or four species, each its own scatter: a `footprint`
  and a higher `priority` on the trees keep them apart and the bushes
  clear of them, `noiseThreshold` gathers each species into groves, and
  og.Grass grows under all of them.
- A scattered mesh draws its coarser levels with distance (the cook's
  levels of detail, a plant's thinned leaves): a copy takes the coarsest
  level whose error is under 2 pixels on screen, and under 8 in shadows,
  reflections and probes, which draw the copies within `distance` that
  the densityMap keeps, in or out of view. A wood of oaks, pines and
  bushes drawn as meshes to 200 m took 7.1 ms of GPU a frame at 720p on an
  M4 Pro against 10.9 with every copy at its full mesh, and 4.9 against
  9.4 with impostors from 30 m. RenderSettings.lodBias moves the levels
  as it does a model's (2 keeps each twice as far, 0 draws full meshes).
  Without compute every copy draws the full mesh.
- A `densityMap` thins copies where its red channel is below 1 (a path
  cleared through flowers): its top row runs along the terrain's -z edge,
  as the control map, loaded linear. It needs GPU culling
  (RenderCapabilities.compute) and is drawn only: placement and footprints
  do not read it.
- og.Grass is generated on the GPU every frame: curved blades on a
  blue-noise pattern fixed to the terrain, in clumps sharing tint and
  height, never over holes. With distance blades thin out, widen to keep
  the ground covered and drop from 15 to 7 to 3 vertices; under and past
  them the terrain shows what the blades show on average (their colours,
  shade and backlight, tinted by `farTint`), so the field keeps its look
  to the horizon with no seam and a shorter `distance` costs less. og.Wind bends them in
  rolling gusts; they are lit like the terrain under them, with light
  through backlit blades (`translucency`), a sheen and shaded roots. A
  field at its defaults is a meadow.
- Up to three more blade kinds (taller reeds, flowers) grow in one field
  as og.GrassType entities; each clump grows one kind, by `typeMap` or by
  the kinds' `share`.
- Scatter and grass grow over the og.Terrain drawing the same data,
  placed as it is; a terrain with `visible` false is only their ground.

## 4. Asset streaming groups

A scene's `stream "<name>" { asset ...; distance x y z r [unload d] | volume "<entity>"; priority n }`
block names assets that load and unload together: glTF models (without
skins or morph targets), textures, texture arrays, flipbooks (an
`array true` sheet brings its array), sounds, environment maps, palettes,
looks and colour LUTs (`.cube`).

- Triggers: the camera within `r` of the anchor (kept until past
  `unload`, 1.25 r by default), the camera inside the named entity's box
  (its position ± half its scale, turned with it), or neither
  (og.stream.request only).
- `og cook` packs each group into `cooked/stream/<name>.ogb`;
  `og build --publish` ships the packs beside the boot bundle, which the
  shell and the service worker fetch first, and fetches a pack only when
  its group loads.
- At every tick boundary the engine picks the wanted groups, admits them by
  priority (then name) within the memory budget (256 MiB by default: GPU
  textures and meshes and decoded audio, as the cook measured them),
  evicting lower-priority groups for a higher one, and puts at most one
  group's assets in place per boundary. A group that cannot fit waits; one
  larger than the whole budget fails.
- Until its group is resident a model draws its proxy (its coarsest LOD,
  untextured), a texture is a 1x1 grey placeholder and a sound plays as
  silence of its own length; then the real ones take over the same
  handles. Physics always collides with the proxy's meshes and a sound's
  length never changes, so the hash is the same streamed or not.
- Texture arrays, flipbooks, environment maps, palettes, looks and LUTs
  have no stand-in: while their group is out the sky lights the scene,
  Stylise.palette quantises to none, PostEffects.lut grades nothing,
  og.render.sprite of the array and applying the look are refused, and a
  sprite, flipbook or particle emitter of the array draws nothing; they
  come back under the same handles. One the frame still uses when its group
  unloads is kept, with a warning in the engine log, until the frame stops
  using it; its bytes count in the budget's `used` and `kept` (and the
  group's StreamStatus.kept), so a group that would not fit beside it waits.
- One queue reads everything: first the chunks of chunked sounds a
  playhead needs soonest (the one playing, then by seconds until it gets
  there at its pitch, a loop's start by the time to its wrap), then the
  loading groups' packs by priority, then name. At most four reads are in
  flight (on the web, fetches; natively a read completes at once, so only
  the order shows).
- StreamEvent arrives at the start of the next tick; og.stream.status and
  og.stream.budget answer at once. `dump_stream` lists the budget, memory
  used and its peak, every group (trigger, state, why, since which tick,
  what it keeps and each asset's bytes), recent changes, the terrain's
  tiles and the fetch queue with each read's path and purpose.

## 5. Cells

Cells stream entities, not assets: an entity with an og.StreamCell and a
sub-scene (a `scene "<id>"` line in its scene block, or StreamCell.scene
from og.load.scene) is a cell.

- When it loads, its sub-scene's entities (with their colliders, bodies and
  everything else) spawn placed by the cell's transform: top-level entities
  are roots at the cell's transform after their own, so physics bodies
  among them are where they look. Each carries an og.CellMember naming the
  cell; on unload every member and its descendants are despawned, wherever
  they moved. Remove og.CellMember to keep an entity past its cell.
- Loading is decided as each tick starts, before any system runs, from
  world state alone: a cell loads while an og.StreamAnchor (a player, a
  vehicle, each player of a networked game) is within StreamCell.load of
  its box (the og.Volume on its entity; without one, the entity's
  position), unloads once every anchor is past StreamCell.unload, and stays
  while `pin` is set. Without any anchor og.Camera's position anchors, for a
  single-player game; a lockstep or rollback session refuses that, since
  each peer's camera is its own.
- Cells load in entity id order and a sub-scene spawns in its text order,
  so the same world spawns the same entity ids on both hosts, in a replay,
  a rollback and after a restore. Which cells are loaded, and their
  entities, are in snapshots, saves and the hash; an unloaded cell's
  entities come back from its sub-scene as authored (unless `persist`).
- A sub-scene's blob is in the boot bundle unless the sub-scene declares a
  cell group (`stream "<name>" { cell; asset ... }`): then it is in that
  group's pack with the assets it names, fetched ahead of the anchors, and
  its models and textures draw their stand-ins until the group is
  resident. A tick that would load a cell whose blob has not landed is
  held, never skipped or reordered: frames go on drawing, the engine log
  records each hold and its length, and `dump_stream`'s `hold` says what
  the tick waits for. Natively a read completes at once, so a native run
  or replay never holds.
- CellEvent is sent in the tick a cell changes, and add hooks and `added`
  filters see its entities arrive. `dump_stream`'s `cells` lists every cell:
  its entity, sub-scene, state, entities and the ticks it last loaded and
  unloaded.
- An og.StreamCell loads a sub-scene's entities near og.StreamAnchor
  entities. A cell's `budget` spreads a large cell over consecutive ticks
  instead of one long one; `loaded` turns true, and CellEvent is sent, in
  the tick its last entity spawns. A cell that stops being wanted before
  then despawns what it spawned and sends nothing.
- `persist` keeps what happened in a cell (moved, damaged, opened; every
  component and name) in the world's cells state when it unloads,
  carried by snapshots, saves and the hash; it comes back as it was
  instead of as authored.
- An anchor's `maxSpeed` reads ahead: a cell's asset group (`cell` in its
  sub-scene's stream block) is fetched once the anchor is within the
  cell's load distance plus reach plus 2 seconds at that speed, so it has
  landed when the cell loads and the tick never waits. It is read only for
  fetching, never by a tick.

## 6. Tilemaps

- A tileset (`type tileset`) gives the grid, solid tiles and collision
  rects, animated tiles and 47-blob or 16-bitmask auto-tiling terrains. A
  map is a hand-written `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`,
  loaded with og.load.tilemap and placed by og.Tilemap; og.tilemap.create
  and og.tilemap.resize make and change maps in game code, saved whole by
  the og.tilemap state block (tilesets by asset id), so a save restores a
  map a session never made.
- Layers draw through the sprite batch in 16x16-tile chunks culled to the
  view, each at the map's sprite layer plus its z, scrolled by its
  parallax. Solid tiles become static colliders merged into rectangles;
  runtime edits (og.tilemap.setTile, re-auto-tiling the neighbourhood) are
  world state and the colliders follow.
- Lighting: with og.Lighting2D on, a layer whose sprite layer is in
  minLayer to maxLayer is lit as sprites there are (ambient plus every
  Light2D, with occluder and castShadow shadows): a torch-lit cave is the
  map, its lights and occluders. A sky or far parallax backdrop opts out
  with a bool custom property `lit` false on the Tiled layer, or `lit false`
  in a tilemap `.ogt`'s layer block (TileLayerInfo.lit reports it).
- Orientations: orthogonal, isometric (diamonds on a turned grid),
  staggered (diamonds in rows) or hexagonal (pointy- or flat-top), from the
  map's `orientation` line or its Tiled map; cellAt, cellCenter and
  neighbours follow it. An isometric or staggered tile's image stands up from its
  cell's bottom, drawn back to front. Solid tiles off the orthogonal grid
  collide as one collider per connected region (a hollow polyline in 2D, a
  prism in 3D; `cellColliders` makes one per cell). Building them happens
  once per edit or resize: in 3D about 75 ms per 400,000 triangles (a
  256 × 256 hex map, half solid at random, release), so keep large 3D hex
  and iso collision layers in large solid regions or edit them rarely.
  Stepping costs per region, not per cell. Terrain layers need a square
  grid (orthogonal or isometric).

## 7. Procedural generation

- Seed every generator from a fork of og.Rng so the main stream does not
  move: `world.resource(og.Rng).value.fork("dungeon").seed`. A fork needs
  og.Rng among a system's reads only.
- The generators (noise, Poisson-disk points, BSP dungeons, wave function
  collapse) draw only from a stream on that seed, in a fixed order, with
  bit-identical arithmetic: the same seed gives the same map natively and
  on the web, in a replay and after a restore. Store the seed, not the map,
  in a save.
- Weighted picks and shuffles are og.Rng's and RngStream's (`weighted`,
  `permutation`; a shuffle is a permutation applied to an array).
  og.tilemap.fill writes a generated grid into a tilemap layer.

## 8. Paths

- A path is measured by arc length: t from 0 to 1 moves at an even speed
  however its points are spaced. `catmullRom` passes through every point
  (three.js CatmullRomCurve3, tension 0.5), `bezier` chains cubic segments
  `p0 c0 c1 p1 c2 c3 p2 ...` (CubicBezierCurve3 in a CurvePath), `linear`
  joins straight segments; a closed path wraps t.
- Declare one in a scene, `path "<name>" { kind catmullRom; closed false; point x y z ... }`,
  and find it with og.path.find, or make one with og.path.create. Uses: a
  timeline camera's `rail "<name>"`, og.PathFollower, og.CameraRail.path,
  og.render.tubeGeometry (TubeGeometryDesc.path).
- Paths are assets, not world state; a hot reload registers them again.

## 9. Navigation and crowds

- Grids (A* and jump point search, eight-way moves that never cut a
  blocked corner, line-of-sight smoothing) suit tile games; flow fields
  suit many agents to one goal; navmeshes suit 3D levels: baked at cook
  time from a level's static colliders and models, or built at run time
  for a flat procedural level, with funnel paths and box obstacles that
  rebuild only the tiles they touch.
- Paths, grid edits, obstacles and flow targets are in snapshots.
- Agents move on the profile's plane: x and y in 2D (a grid's rows run
  up), x and z in 3D (y kept on grids, from the surface on a navmesh).
- og.Crowd adds reciprocal avoidance (ORCA): each tick an agent takes the
  velocity nearest what its path wants that keeps it clear of its nearest
  agents for `horizon` seconds and of walls for `obstacleHorizon`. Agents
  meeting perfectly symmetrically (a ring swapping places) are kept from
  locking by a margin of up to 5 % of the radius picked from the entity, so
  add no jitter of your own. Without it an agent has only og.Steering's
  separation.

## 10. AI behaviours

- Behaviour trees and state machines are `type behaviour` `.ogt` files;
  perception gives sight cones checked against physics occlusion and
  hearing of spatial sounds and game noises; each agent has a blackboard.
- Trees use BehaviorTree.CPP's control names. ReactiveFallback rechecks
  priority from its first child each think; ReactiveSequence rechecks its
  conditions from its first child; Sequence resumes its running child but
  restarts after a failure or completion (it is not SequenceWithMemory). A
  higher-priority branch halts the lower running one and resets its
  sequence cursor; game actions receive AI_INTERRUPTED. Inverter flips
  success and failure, ForceSuccess turns either into success; both keep
  running. Command cooldowns are explicit engine state.
- A machine's states run entry and exit action blocks. Machines are tick machines, not XState statecharts: at most one
  transition per think, exit then entry (also on a self-transition), then
  the destination's body. Initial entry runs once and a restore keeps it.
  There are no queued events, parallel states or actors.
- A running action moves the agent through its og.NavAgent, or runs one of
  the game's commands (GameDesc.commands, with the agent's entity as its
  first argument) before the next tick.
- Conditions read blackboard keys (declared, set by actions or og.ai.set,
  or built in: self, position, sees, heard, target, targetPosition,
  targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the
  agent's component fields (`Health.hp`) and resource fields
  (`Game.alarm`), and compare with a literal or another key or field read
  each time (`check Foe.hp below Tuning.fleeHp`): keep thresholds in a
  tuning resource so they are tuned without editing behaviours.
- og.ai.restart starts a behaviour afresh. `dump_ai:<entity>` writes an
  agent's state, running node and blackboard: read it before guessing why
  an agent stands still.

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `stream` module

Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance x y z r [unload d] | volume "<entity>"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads.

At every tick boundary the engine decides which groups are wanted, admits them by priority (then name) within the memory budget (256 MiB by default; GPU textures and meshes and decoded audio, as the cook measured them), evicting lower-priority resident groups to make room for a higher one, and reads and puts in place at most one group's assets per boundary. A group that cannot fit waits, queued; one larger than the whole budget fails. Until its group is resident, a model draws its proxy (its coarsest LOD, untextured), a texture is a 1x1 grey placeholder and a sound plays as silence of its own length; then the real ones take over the same handles. Physics always collides with the proxy's meshes and a sound's length never changes, so nothing a tick reads depends on what is resident: a run's world hash is the same streamed or not.

Texture arrays, flipbooks, environment maps, palettes, looks and colour LUTs have no stand-in: they load once their group is resident, and when it unloads the renderer lets them go and their handles wait (the sky lights the scene, Stylise.palette quantises to none, PostEffects.lut grades nothing, og.render.sprite of the array and applying the look are refused, an og.Sprite, a flipbook's sprite or a particle emitter of the array draws nothing) until the group is resident again, when they come back under the same handles. One the frame still uses when its group unloads (the map Environment lights it with, the palette Stylise quantises to, an array a sprite, a screen sprite or a particle emitter draws from, the look last applied, the LUT PostEffects grades with) is never freed under the frame: it is kept, with a warning naming it in the engine's log, until the frame stops using it. What is kept still takes its memory: its bytes count in the budget's `used` (and `kept`, and its group's StreamStatus.kept) until it is let go, so a group that would not fit beside it waits.

Every change of a group's state sends StreamEvent at the start of the next tick; og.stream.status and og.stream.budget answer at once, and the probe's `dump_stream` lists the budget, the memory used and its peak, every group (trigger, state, why, since which tick, what it keeps for the frame after unloading with each asset's bytes), the recent changes, the terrain streamer's tiles and the fetch queue.

What streams is read through one queue: first the chunks of the sounds cooked in chunks that a playhead needs soonest (the one it is playing, then by the seconds until it gets there at its pitch, a loop's start by the time to its wrap), then the loading groups' packs by priority (then name). At most four reads are in flight at once (on the web, fetches; natively a read completes at once, so only the order shows); the rest wait in that order. `dump_stream`'s `fetch` lists what is in flight and what waits, each with its path and what it is for (a sound's chunk and its seconds, a group's pack and its priority).

World cells (G3-3) stream entities, not assets: an entity with an og.StreamCell and a sub-scene (a `scene "<id>"` line in its scene block, or StreamCell.scene from og.load.scene) is a cell. Its sub-scene's entities, and with them their colliders, bodies and everything else they carry, spawn when it loads, placed by the cell's transform (its top-level entities are roots, at the cell's transform after their own, so physics bodies among them are where they look) and each with an og.CellMember naming the cell; when it unloads, every member and its descendants are despawned. Whether a cell is loaded is decided as each tick starts, before any system runs, from world state alone: it loads while an og.StreamAnchor (a player, a vehicle) is within StreamCell.load of its box, unloads once every anchor is past StreamCell.unload, and stays loaded while StreamCell.pin is set. Without any anchor the og.Camera resource's position anchors, a convenience for a single-player game; a lockstep or rollback session (og.net) refuses that, since each peer's camera is its own. Cells load in entity id order and a sub-scene's entities spawn in its text order, so the same world spawns the same entity ids on both hosts, in a replay, a rollback and after a restore. A sub-scene's blob is in the boot bundle unless the sub-scene declares a cell group (`stream "<name>" { cell; asset ... }`): then the blob is in that group's pack with the assets it names, fetched ahead of the anchors (StreamAnchor.maxSpeed), and its models and textures draw their stand-ins until the group is resident. A tick that would load a cell whose blob has not landed is held, never skipped or reordered: frames go on drawing, the engine's log records each hold and how long it lasted, and `dump_stream`'s `hold` says what the tick waits for. Natively a read completes at once, so a native run or replay never holds.

A cell's state is world state: StreamCell.loaded, and its sub-scene's entities, are in snapshots, saves and the world hash, so restoring a save restores which cells are loaded and what is in them; an unloaded cell's entities are not in the world and come back from its sub-scene, as authored, when it loads again. Each change sends CellEvent in the tick it happens, and add hooks and `added` filters on the sub-scene's components see its entities arrive. The probe's `dump_stream` lists every cell (`cells`): its entity, sub-scene, state, entities and the ticks it last loaded and unloaded.

- **TypeScript**: functions are the namespace `og.stream`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module stream {
  doc "Asset streaming groups (phase 3): a scene's `stream \"<name>\" { asset ...; distance x y z r [unload d] | volume \"<entity>\"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads."
  doc "At every tick boundary the engine decides which groups are wanted, admits them by priority (then name) within the memory budget (256 MiB by default; GPU textures and meshes and decoded audio, as the cook measured them), evicting lower-priority resident groups to make room for a higher one, and reads and puts in place at most one group's assets per boundary. A group that cannot fit waits, queued; one larger than the whole budget fails. Until its group is resident, a model draws its proxy (its coarsest LOD, untextured), a texture is a 1x1 grey placeholder and a sound plays as silence of its own length; then the real ones take over the same handles. Physics always collides with the proxy's meshes and a sound's length never changes, so nothing a tick reads depends on what is resident: a run's world hash is the same streamed or not."
  doc "Texture arrays, flipbooks, environment maps, palettes, looks and colour LUTs have no stand-in: they load once their group is resident, and when it unloads the renderer lets them go and their handles wait (the sky lights the scene, Stylise.palette quantises to none, PostEffects.lut grades nothing, og.render.sprite of the array and applying the look are refused, an og.Sprite, a flipbook's sprite or a particle emitter of the array draws nothing) until the group is resident again, when they come back under the same handles. One the frame still uses when its group unloads (the map Environment lights it with, the palette Stylise quantises to, an array a sprite, a screen sprite or a particle emitter draws from, the look last applied, the LUT PostEffects grades with) is never freed under the frame: it is kept, with a warning naming it in the engine's log, until the frame stops using it. What is kept still takes its memory: its bytes count in the budget's `used` (and `kept`, and its group's StreamStatus.kept) until it is let go, so a group that would not fit beside it waits."
  doc "Every change of a group's state sends StreamEvent at the start of the next tick; og.stream.status and og.stream.budget answer at once, and the probe's `dump_stream` lists the budget, the memory used and its peak, every group (trigger, state, why, since which tick, what it keeps for the frame after unloading with each asset's bytes), the recent changes, the terrain streamer's tiles and the fetch queue."
  doc "What streams is read through one queue: first the chunks of the sounds cooked in chunks that a playhead needs soonest (the one it is playing, then by the seconds until it gets there at its pitch, a loop's start by the time to its wrap), then the loading groups' packs by priority (then name). At most four reads are in flight at once (on the web, fetches; natively a read completes at once, so only the order shows); the rest wait in that order. `dump_stream`'s `fetch` lists what is in flight and what waits, each with its path and what it is for (a sound's chunk and its seconds, a group's pack and its priority)."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 25499</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.StreamCell`](stream/record-StreamCell.md) | component |  | A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). |
| [`og.StreamAnchor`](stream/record-StreamAnchor.md) | component |  | What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. |
| [`og.CellMember`](stream/record-CellMember.md) | component |  | Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. |
| [`og.CellEvent`](stream/record-CellEvent.md) | event |  | A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned). |
| [`StreamState`](stream/type-StreamState.md) | enum |  | Where a streaming group is. |
| [`STREAM_UNLOADED`](#const-STREAM_UNLOADED) | constant |  | `StreamEvent.state`: the group is unloaded (StreamState unloaded). |
| [`STREAM_QUEUED`](#const-STREAM_QUEUED) | constant |  | `StreamEvent.state`: the group waits for room in the budget (StreamState queued). |
| [`STREAM_LOADING`](#const-STREAM_LOADING) | constant |  | `StreamEvent.state`: the group's pack is being read (StreamState loading). |
| [`STREAM_RESIDENT`](#const-STREAM_RESIDENT) | constant |  | `StreamEvent.state`: the group's assets are in place (StreamState resident). |
| [`STREAM_FAILED`](#const-STREAM_FAILED) | constant |  | `StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why). |
| [`og.StreamEvent`](stream/record-StreamEvent.md) | event |  | A group changed state: sent by og.stream at the start of the tick after the change. |
| [`StreamStatus`](stream/type-StreamStatus.md) | struct |  | One streaming group. |
| [`StreamBudget`](stream/type-StreamBudget.md) | struct |  | The streaming memory budget and every group. |
| [`og.stream.status`](stream/fn-status.md) | function | `(world: World, group: string): Result<StreamStatus>` | One streaming group's state. |
| [`og.stream.request`](stream/fn-request.md) | function | `(world: World, group: string): Result<void>` | Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority. |
| [`og.stream.release`](stream/fn-release.md) | function | `(world: World, group: string): Result<void>` | Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it. |
| [`og.stream.budget`](stream/fn-budget.md) | function | `(world: World): Result<StreamBudget>` | The memory budget, what the groups take now, the peak and every group. |
| [`og.stream.setBudget`](stream/fn-setBudget.md) | function | `(world: World, bytes: number): Result<void>` | Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit. |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-STREAM_UNLOADED"></a>`STREAM_UNLOADED` | `number` | `0` | `StreamEvent.state`: the group is unloaded (StreamState unloaded). |
| <a id="const-STREAM_QUEUED"></a>`STREAM_QUEUED` | `number` | `1` | `StreamEvent.state`: the group waits for room in the budget (StreamState queued). |
| <a id="const-STREAM_LOADING"></a>`STREAM_LOADING` | `number` | `2` | `StreamEvent.state`: the group's pack is being read (StreamState loading). |
| <a id="const-STREAM_RESIDENT"></a>`STREAM_RESIDENT` | `number` | `3` | `StreamEvent.state`: the group's assets are in place (StreamState resident). |
| <a id="const-STREAM_FAILED"></a>`STREAM_FAILED` | `number` | `4` | `StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why). |

```ogt
const STREAM_UNLOADED u32 0 "`StreamEvent.state`: the group is unloaded (StreamState unloaded)."
```

<sub>`api.ogt` line 25566</sub>

```ogt
const STREAM_QUEUED u32 1 "`StreamEvent.state`: the group waits for room in the budget (StreamState queued)."
```

<sub>`api.ogt` line 25567</sub>

```ogt
const STREAM_LOADING u32 2 "`StreamEvent.state`: the group's pack is being read (StreamState loading)."
```

<sub>`api.ogt` line 25568</sub>

```ogt
const STREAM_RESIDENT u32 3 "`StreamEvent.state`: the group's assets are in place (StreamState resident)."
```

<sub>`api.ogt` line 25569</sub>

```ogt
const STREAM_FAILED u32 4 "`StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why)."
```

<sub>`api.ogt` line 25570</sub>

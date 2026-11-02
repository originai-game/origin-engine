<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `stream` module

Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance x y z r [unload d] | volume "<entity>"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads.

At every tick boundary the engine decides which groups are wanted, admits them by priority (then name) within the memory budget (256 MiB by default; GPU textures and meshes and decoded audio, as the cook measured them), evicting lower-priority resident groups to make room for a higher one, and reads and puts in place at most one group's assets per boundary. A group that cannot fit waits, queued; one larger than the whole budget fails. Until its group is resident, a model draws its proxy (its coarsest LOD, untextured), a texture is a 1x1 grey placeholder and a sound plays as silence of its own length; then the real ones take over the same handles. Physics always collides with the proxy's meshes and a sound's length never changes, so nothing a tick reads depends on what is resident: a run's world hash is the same streamed or not.

Texture arrays, flipbooks, environment maps, palettes, looks and colour LUTs have no stand-in: they load once their group is resident, and when it unloads the renderer lets them go and their handles wait (the sky lights the scene, Stylise.palette quantises to none, PostEffects.lut grades nothing, og.render.sprite of the array and applying the look are refused, an og.Sprite, a flipbook's sprite or a particle emitter of the array draws nothing) until the group is resident again, when they come back under the same handles. One the frame still uses when its group unloads (the map Environment lights it with, the palette Stylise quantises to, an array a sprite, a screen sprite or a particle emitter draws from, the look last applied, the LUT PostEffects grades with) is never freed under the frame: it is kept, with a warning naming it in the engine's log, until the frame stops using it. What is kept still takes its memory: its bytes count in the budget's `used` (and `kept`, and its group's StreamStatus.kept) until it is let go, so a group that would not fit beside it waits.

Every change of a group's state sends StreamEvent at the start of the next tick; og.stream.status and og.stream.budget answer at once, and the probe's `dump_stream` lists the budget, the memory used and its peak, every group (trigger, state, why, since which tick, what it keeps for the frame after unloading with each asset's bytes), the recent changes, the terrain streamer's tiles and the fetch queue.

What streams is read through one queue: first the chunks of the sounds cooked in chunks that a playhead needs soonest (the one it is playing, then by the seconds until it gets there at its pitch, a loop's start by the time to its wrap), then the loading groups' packs by priority (then name). At most four reads are in flight at once (on the web, fetches; natively a read completes at once, so only the order shows); the rest wait in that order. `dump_stream`'s `fetch` lists what is in flight and what waits, each with its path and what it is for (a sound's chunk and its seconds, a group's pack and its priority).

World cells (G3-3) stream entities, not assets: an entity with an og.StreamCell and a sub-scene (a `scene "<id>"` line in its scene block, or StreamCell.scene from og.load.scene) is a cell. Its sub-scene's entities, and with them their colliders, bodies and everything else they carry, spawn when it loads, placed by the cell's transform (its top-level entities are roots, at the cell's transform after their own, so physics bodies among them are where they look) and each with an og.CellMember naming the cell; when it unloads, every member and its descendants are despawned. Whether a cell is loaded is decided as each tick starts, before any system runs, from world state alone: it loads while an og.StreamAnchor (a player, a vehicle) is within StreamCell.load of its box, unloads once every anchor is past StreamCell.unload, and stays loaded while StreamCell.pin is set. Without any anchor the og.Camera resource's position anchors, a convenience for a single-player game; a lockstep or rollback session (og.net) refuses that, since each peer's camera is its own. Cells load in entity id order and a sub-scene's entities spawn in its text order, so the same world spawns the same entity ids on both hosts, in a replay, a rollback and after a restore. Nothing about a cell waits on a download: the sub-scene's blob is in the boot bundle, and its models and textures draw their stand-ins until their streaming group is resident.

A cell's state is world state: StreamCell.loaded, and its sub-scene's entities, are in snapshots, saves and the world hash, so restoring a save restores which cells are loaded and what is in them; an unloaded cell's entities are not in the world and come back from its sub-scene, as authored, when it loads again. Each change sends CellEvent in the tick it happens, and add hooks and `added` filters on the sub-scene's components see its entities arrive. The probe's `dump_stream` lists every cell (`cells`): its entity, sub-scene, state, entities and the ticks it last loaded and unloaded.

- **TypeScript**: functions are the namespace `og.stream`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module stream {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 26403</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.StreamCell`](#record-StreamCell) | component | A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). The box is the entity's world position ± half `size`, turned with it; a zero size is a point. |
| [`og.StreamAnchor`](#record-StreamAnchor) | component | What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. Without any anchor the og.Camera resource's position anchors (refused in an og.net session). |
| [`og.CellMember`](#record-CellMember) | component | Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. Remove it from an entity to keep it past its cell's unload (an item the player picked up); the cell spawns the item again from its sub-scene when it loads again. |
| [`og.CellEvent`](#record-CellEvent) | event | A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned). |
| [`StreamState`](#type-StreamState) | enum | Where a streaming group is. |
| [`STREAM_UNLOADED`](#const-STREAM_UNLOADED) | constant | `StreamEvent.state`: the group is unloaded (StreamState unloaded). |
| [`STREAM_QUEUED`](#const-STREAM_QUEUED) | constant | `StreamEvent.state`: the group waits for room in the budget (StreamState queued). |
| [`STREAM_LOADING`](#const-STREAM_LOADING) | constant | `StreamEvent.state`: the group's pack is being read (StreamState loading). |
| [`STREAM_RESIDENT`](#const-STREAM_RESIDENT) | constant | `StreamEvent.state`: the group's assets are in place (StreamState resident). |
| [`STREAM_FAILED`](#const-STREAM_FAILED) | constant | `StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why). |
| [`og.StreamEvent`](#record-StreamEvent) | event | A group changed state: sent by og.stream at the start of the tick after the change. |
| [`StreamStatus`](#type-StreamStatus) | struct | One streaming group. |
| [`StreamBudget`](#type-StreamBudget) | struct | The streaming memory budget and every group. |
| [`og.stream.status`](#fn-status) | function | One streaming group's state. |
| [`og.stream.request`](#fn-request) | function | Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority. |
| [`og.stream.release`](#fn-release) | function | Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it. |
| [`og.stream.budget`](#fn-budget) | function | The memory budget, what the groups take now, the peak and every group. |
| [`og.stream.setBudget`](#fn-setBudget) | function | Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit. |

## Functions

<a id="fn-status"></a>

### `og.stream.status`

One streaming group's state.

```ts
og.stream.status(world: World, group: string): Result<StreamStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name, as its scene declares it. |

**Returns** `Result<StreamStatus>`: Refused for a group no scene declares.

Binding symbol `og_stream_status` · publish builds: kept

```ogt
fn status {
  doc "One streaming group's state."
  param world World "The world."
  param group String "The group's name, as its scene declares it."
  returns Result.StreamStatus "Refused for a group no scene declares."
}
```

<sub>`api.ogt` line 26518</sub>

<a id="fn-request"></a>

### `og.stream.request`

Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority.

```ts
og.stream.request(world: World, group: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name. |

**Returns** `Result<void>`: Refused for a group no scene declares.

Binding symbol `og_stream_request` · publish builds: kept

```ogt
fn request {
  doc "Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority."
  param world World "The world."
  param group String "The group's name."
  returns Result.Void "Refused for a group no scene declares."
}
```

<sub>`api.ogt` line 26525</sub>

<a id="fn-release"></a>

### `og.stream.release`

Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it.

```ts
og.stream.release(world: World, group: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name. |

**Returns** `Result<void>`: Refused for a group no scene declares.

Binding symbol `og_stream_release` · publish builds: kept

```ogt
fn release {
  doc "Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it."
  param world World "The world."
  param group String "The group's name."
  returns Result.Void "Refused for a group no scene declares."
}
```

<sub>`api.ogt` line 26532</sub>

<a id="fn-budget"></a>

### `og.stream.budget`

The memory budget, what the groups take now, the peak and every group.

```ts
og.stream.budget(world: World): Result<StreamBudget>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<StreamBudget>`: The budget and the groups; refused only for a world that does not exist.

Binding symbol `og_stream_budget` · publish builds: kept

```ogt
fn budget {
  doc "The memory budget, what the groups take now, the peak and every group."
  param world World "The world."
  returns Result.StreamBudget "The budget and the groups; refused only for a world that does not exist."
}
```

<sub>`api.ogt` line 26539</sub>

<a id="fn-setBudget"></a>

### `og.stream.setBudget`

Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit.

```ts
og.stream.setBudget(world: World, bytes: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `bytes` | `number` | no | The bytes resident groups may take. |

**Returns** `Result<void>`: Refused for a negative or non-finite size.

Binding symbol `og_stream_set_budget` · publish builds: kept

```ogt
fn setBudget {
  doc "Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit."
  param world World "The world."
  param bytes f64 "The bytes resident groups may take."
  returns Result.Void "Refused for a negative or non-finite size."
}
```

<sub>`api.ogt` line 26545</sub>

## Components, resources and events

<a id="record-StreamCell"></a>

### `og.StreamCell` (component)

A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). The box is the entity's world position ± half `size`, turned with it; a zero size is a point.

Token `og.StreamCell`, registered under the schema name `og.StreamCell`; its value type is `StreamCell`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `Asset` | `0` | — | — | — | The sub-scene it loads: set from the entity's `scene "<id>"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads). |
| `size` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The box's full extents, in metres along the entity's axes (its scale does not apply). Zero: a point at the entity's position. |
| `load` | `number` | `0` | ≥ 0 | `m` | — | Loads while an anchor is within this distance of the box (0: only while one is inside it). |
| `unload` | `number` | `0` | ≥ 0 | `m` | — | Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`. |
| `pin` | `boolean` | `false` | — | — | — | Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again. |
| `loaded` | `boolean` | `false` | — | — | read only | Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded). |

```ogt
component StreamCell {
  doc "A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). The box is the entity's world position ± half `size`, turned with it; a zero size is a point."
  field scene Asset "The sub-scene it loads: set from the entity's `scene \"<id>\"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads)."
  field size Vector3 "The box's full extents, in metres along the entity's axes (its scale does not apply). Zero: a point at the entity's position." {
    unit m
  }
  field load f32 "Loads while an anchor is within this distance of the box (0: only while one is inside it)." {
    unit m
    min 0
  }
  field unload f32 "Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`." {
    unit m
    min 0
  }
  field pin bool "Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again."
  field loaded bool "Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 26414</sub>

<a id="record-StreamAnchor"></a>

### `og.StreamAnchor` (component)

What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. Without any anchor the og.Camera resource's position anchors (refused in an og.net session).

Token `og.StreamAnchor`, registered under the schema name `og.StreamAnchor`; its value type is `StreamAnchor`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reach` | `number` | `0` | ≥ 0 | `m` | — | Added to every cell's load and unload distances for this anchor: a fast vehicle reaches further than a walker. |

```ogt
component StreamAnchor {
  doc "What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. Without any anchor the og.Camera resource's position anchors (refused in an og.net session)."
  field reach f32 "Added to every cell's load and unload distances for this anchor: a fast vehicle reaches further than a walker." {
    unit m
    min 0
  }
}
```

<sub>`api.ogt` line 26434</sub>

<a id="record-CellMember"></a>

### `og.CellMember` (component)

Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. Remove it from an entity to keep it past its cell's unload (an item the player picked up); the cell spawns the item again from its sub-scene when it loads again.

Token `og.CellMember`, registered under the schema name `og.CellMember`; its value type is `CellMember`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `Entity` | `0` | — | — | read only | The cell's entity. |

```ogt
component CellMember {
  doc "Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. Remove it from an entity to keep it past its cell's unload (an item the player picked up); the cell spawns the item again from its sub-scene when it loads again."
  field cell Entity "The cell's entity." {
    readonly true
  }
}
```

<sub>`api.ogt` line 26442</sub>

<a id="record-CellEvent"></a>

### `og.CellEvent` (event)

A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned).

Token `og.CellEvent`, registered under the schema name `og.CellEvent`; its value type is `CellEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `Entity` | `0` | — | — | — | The cell's entity. |
| `loaded` | `boolean` | `false` | — | — | — | True: its sub-scene's entities spawned; false: its members and their descendants were despawned. |
| `entities` | `number` | `0` | — | — | — | How many entities spawned or were despawned. |

```ogt
event CellEvent {
  doc "A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned)."
  field cell Entity "The cell's entity."
  field loaded bool "True: its sub-scene's entities spawned; false: its members and their descendants were despawned."
  field entities u32 "How many entities spawned or were despawned."
}
```

<sub>`api.ogt` line 26449</sub>

<a id="record-StreamEvent"></a>

### `og.StreamEvent` (event)

A group changed state: sent by og.stream at the start of the tick after the change.

Token `og.StreamEvent`, registered under the schema name `og.StreamEvent`; its value type is `StreamEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group's name (at most 63 bytes). |
| `state` | `number` | `0` | — | — | — | Its new state: STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT or STREAM_FAILED (StreamState's order). |

```ogt
event StreamEvent {
  doc "A group changed state: sent by og.stream at the start of the tick after the change."
  field group String "The group's name (at most 63 bytes)."
  field state u32 "Its new state: STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT or STREAM_FAILED (StreamState's order)."
}
```

<sub>`api.ogt` line 26471</sub>

## Types

<a id="type-StreamState"></a>

### `StreamState` (enum)

Where a streaming group is.

A string: one of the variants below. The zero value is `"unloaded"`.

| Variant | Doc |
|---|---|
| `"unloaded"` | Not wanted; its assets draw and play their stand-ins. |
| `"queued"` | Wanted, waiting for room in the memory budget. |
| `"loading"` | Admitted: its pack is being read (its memory already counts). |
| `"resident"` | Its assets are in place of their stand-ins. |
| `"failed"` | Its pack could not be read, or it alone needs more than the budget (StreamStatus.reason says which). |

```ogt
enum StreamState {
  doc "Where a streaming group is."
  variant unloaded "Not wanted; its assets draw and play their stand-ins."
  variant queued "Wanted, waiting for room in the memory budget."
  variant loading "Admitted: its pack is being read (its memory already counts)."
  variant resident "Its assets are in place of their stand-ins."
  variant failed "Its pack could not be read, or it alone needs more than the budget (StreamStatus.reason says which)."
}
```

<sub>`api.ogt` line 26456</sub>

<a id="type-StreamStatus"></a>

### `StreamStatus` (struct)

One streaming group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `scene` | `string` | `""` | — | — | — | The scene that declares it. |
| `state` | `StreamState` | `"unloaded"` | — | — | — | Where it is. |
| `wanted` | `boolean` | `false` | — | — | — | Whether its trigger or a request wants it now. |
| `requested` | `boolean` | `false` | — | — | — | Whether og.stream.request asked for it (until og.stream.release). |
| `priority` | `number` | `0` | — | — | — | Higher loads first and is evicted last. |
| `trigger` | `string` | `""` | — | — | — | manual, distance or volume. |
| `memory` | `number` | `0` | — | `bytes` | — | What its assets take once loaded, in bytes. |
| `bytes` | `number` | `0` | — | `bytes` | — | Its pack's size, in bytes. |
| `assets` | `readonly string[]` | `[]` | — | — | — | Its asset ids. |
| `reason` | `string` | `""` | — | — | — | Why it is queued or failed; empty otherwise. |
| `since` | `number` | `0` | — | — | — | The tick of its last change. |
| `loads` | `number` | `0` | — | — | — | How many times it became resident. |
| `kept` | `number` | `0` | — | `bytes` | — | The bytes of its assets kept for the frame after it unloaded (counted in StreamBudget.used until let go); 0 otherwise. |

```ogt
struct StreamStatus {
  doc "One streaming group."
  field name String "Its name."
  field scene String "The scene that declares it."
  field state StreamState "Where it is."
  field wanted bool "Whether its trigger or a request wants it now."
  field requested bool "Whether og.stream.request asked for it (until og.stream.release)."
  field priority i32 "Higher loads first and is evicted last."
  field trigger String "manual, distance or volume."
  field memory f64 "What its assets take once loaded, in bytes." {
    unit bytes
  }
  field bytes f64 "Its pack's size, in bytes." {
    unit bytes
  }
  field assets List.String "Its asset ids."
  field reason String "Why it is queued or failed; empty otherwise."
  field since f64 "The tick of its last change."
  field loads u32 "How many times it became resident."
  field kept f64 "The bytes of its assets kept for the frame after it unloaded (counted in StreamBudget.used until let go); 0 otherwise." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 26477</sub>

<a id="type-StreamBudget"></a>

### `StreamBudget` (struct)

The streaming memory budget and every group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `budget` | `number` | `0` | — | `bytes` | — | The bytes resident and loading groups may take. |
| `used` | `number` | `0` | — | `bytes` | — | The bytes they take now, with what unloaded groups keep for the frame. |
| `kept` | `number` | `0` | — | `bytes` | — | Of `used`, the bytes of assets kept for the frame after their group unloaded (an array a sprite still draws, the map the frame is lit by, ...). |
| `peak` | `number` | `0` | — | `bytes` | — | The most they ever took (never above the budget at the time). |
| `groups` | `readonly StreamStatus[]` | `[]` | — | — | — | Every group, by name. |

```ogt
struct StreamBudget {
  doc "The streaming memory budget and every group."
  field budget f64 "The bytes resident and loading groups may take." {
    unit bytes
  }
  field used f64 "The bytes they take now, with what unloaded groups keep for the frame." {
    unit bytes
  }
  field kept f64 "Of `used`, the bytes of assets kept for the frame after their group unloaded (an array a sprite still draws, the map the frame is lit by, ...)." {
    unit bytes
  }
  field peak f64 "The most they ever took (never above the budget at the time)." {
    unit bytes
  }
  field groups List.StreamStatus "Every group, by name."
}
```

<sub>`api.ogt` line 26501</sub>

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

<sub>`api.ogt` line 26465</sub>

```ogt
const STREAM_QUEUED u32 1 "`StreamEvent.state`: the group waits for room in the budget (StreamState queued)."
```

<sub>`api.ogt` line 26466</sub>

```ogt
const STREAM_LOADING u32 2 "`StreamEvent.state`: the group's pack is being read (StreamState loading)."
```

<sub>`api.ogt` line 26467</sub>

```ogt
const STREAM_RESIDENT u32 3 "`StreamEvent.state`: the group's assets are in place (StreamState resident)."
```

<sub>`api.ogt` line 26468</sub>

```ogt
const STREAM_FAILED u32 4 "`StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why)."
```

<sub>`api.ogt` line 26469</sub>

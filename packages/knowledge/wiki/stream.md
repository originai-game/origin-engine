<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `stream` module

Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance ... | volume ...; priority n }` blocks name assets that load and unload together, one pack each, admitted by priority within a memory budget (256 MiB by default). Until resident, models draw proxies, textures a placeholder and sounds silence of their own length, so nothing a tick reads depends on what is resident.

World cells (og.StreamCell) stream entities: a cell's sub-scene spawns while an og.StreamAnchor is near and despawns when none is, decided from world state as each tick starts; its state is world state.

See `world §4`.

- **TypeScript**: functions are the namespace `og.stream`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module stream {
  doc "Asset streaming groups (phase 3): a scene's `stream \"<name>\" { asset ...; distance ... | volume ...; priority n }` blocks name assets that load and unload together, one pack each, admitted by priority within a memory budget (256 MiB by default). Until resident, models draw proxies, textures a placeholder and sounds silence of their own length, so nothing a tick reads depends on what is resident."
  doc "World cells (og.StreamCell) stream entities: a cell's sub-scene spawns while an og.StreamAnchor is near and despawns when none is, decided from world state as each tick starts; its state is world state."
  # declarations follow, each on its own page
```

<sub>Module `stream` line 1</sub>

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

<sub>Module `stream` line 66</sub>

```ogt
const STREAM_QUEUED u32 1 "`StreamEvent.state`: the group waits for room in the budget (StreamState queued)."
```

<sub>Module `stream` line 67</sub>

```ogt
const STREAM_LOADING u32 2 "`StreamEvent.state`: the group's pack is being read (StreamState loading)."
```

<sub>Module `stream` line 68</sub>

```ogt
const STREAM_RESIDENT u32 3 "`StreamEvent.state`: the group's assets are in place (StreamState resident)."
```

<sub>Module `stream` line 69</sub>

```ogt
const STREAM_FAILED u32 4 "`StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why)."
```

<sub>Module `stream` line 70</sub>

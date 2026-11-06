<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `probe` module

The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members.

In a page it is published on globalThis.__runtime_probe__; on native, `og` serves it as JSON-RPC 2.0 over a loopback WebSocket at /probe.

A refusal is false or null exactly as protocol 5; a JSON-RPC error is only for a malformed call or an unknown method.

Probe entity ids are entity names; an unnamed entity is `#<index>.<generation>`.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module probe {
  doc "The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members."
  doc "In a page it is published on globalThis.__runtime_probe__; on native, `og` serves it as JSON-RPC 2.0 over a loopback WebSocket at /probe."
  doc "A refusal is false or null exactly as protocol 5; a JSON-RPC error is only for a malformed call or an unknown method."
  doc "Probe entity ids are entity names; an unnamed entity is `#<index>.<generation>`."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 23946</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`PROBE_GLOBAL`](#const-PROBE_GLOBAL) | constant |  | The global the page publishes the probe on. |
| [`PROBE_VERSION`](#const-PROBE_VERSION) | constant |  | The RuntimeProbe contract revision both sides speak. |
| [`PROBE_WIRE_VERSION`](#const-PROBE_WIRE_VERSION) | constant |  | The wire revision hello reports. |
| [`REPLAY_SEED_PARAM`](#const-REPLAY_SEED_PARAM) | constant |  | The page query parameter that starts a deterministic replay from a uint32 seed. |
| [`PLAYTEST_MAX_TICKS`](#const-PLAYTEST_MAX_TICKS) | constant |  | The longest playtest one request may advance, in fixed steps. |
| [`PROBE_PROGRAM_MAX_OPERATIONS`](#const-PROBE_PROGRAM_MAX_OPERATIONS) | constant |  | The most operations one probe program may carry. |
| [`PickResult`](probe/type-PickResult.md) | struct |  | What a screen point resolved to. |
| [`ProbeTunable`](probe/type-ProbeTunable.md) | struct |  | One published tunable parameter and the range it means something over. |
| [`ProbeEntityRef`](probe/type-ProbeEntityRef.md) | struct |  | One entity the probe publishes in state(). |
| [`ProbeTimings`](probe/type-ProbeTimings.md) | struct |  | The fixed timings head of state(). |
| [`ProbeTiming`](probe/type-ProbeTiming.md) | struct |  | One named span of the last tick or frame. |
| [`ProbeState`](probe/type-ProbeState.md) | struct |  | The probe's state(): a fixed head, then the game's own facts. |
| [`AdvanceOperation`](probe/type-AdvanceOperation.md) | struct |  | The advance operation's payload. |
| [`InputOperation`](probe/type-InputOperation.md) | struct |  | The input operation's payload. |
| [`PickOperation`](probe/type-PickOperation.md) | struct |  | The pick operation's payload. |
| [`EntityOperation`](probe/type-EntityOperation.md) | struct |  | The entity operation's payload. |
| [`ExpectOperation`](probe/type-ExpectOperation.md) | struct |  | The expect operation's payload. |
| [`ProbeOperation`](probe/type-ProbeOperation.md) | union |  | One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip. |
| [`ProbeProgramResult`](probe/type-ProbeProgramResult.md) | struct |  | What a program observed: one answer per operation that ran. |
| [`Hello`](probe/type-Hello.md) | struct |  | The wire's hello: who is speaking. |
| [`BuildProfile`](probe/type-BuildProfile.md) | enum |  | The build configuration; publish builds keep only the player-facing probe members. |
| [`Capture`](probe/type-Capture.md) | struct |  | A captured frame. |
| [`Heard`](probe/type-Heard.md) | struct |  | What the host's mixer rendered last: the audio up to the current tick. |
| [`ProbeHeap`](probe/type-ProbeHeap.md) | struct |  | The engine's allocator, counted (both hosts install a counting global allocator). |
| [`ProbeWorldMemory`](probe/type-ProbeWorldMemory.md) | struct |  | The world's size. |
| [`ProbeStreamMemory`](probe/type-ProbeStreamMemory.md) | struct |  | The streaming memory budget (og.stream.budget): what the admitted groups' assets take. |
| [`ProbeMemory`](probe/type-ProbeMemory.md) | struct |  | What the engine holds, in one shape on both hosts; a figure a host cannot read is null. |
| [`SweepDesc`](probe/type-SweepDesc.md) | desc struct |  | A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture. |
| [`SweepFrame`](probe/type-SweepFrame.md) | struct |  | One frame of a sweep. |
| [`SweepResult`](probe/type-SweepResult.md) | struct |  | A sweep's frames, one per value. |
| [`ReloadReport`](probe/type-ReloadReport.md) | struct |  | What a hot reload did. |
| [`SnapshotInfo`](probe/type-SnapshotInfo.md) | struct |  | A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process. |
| [`DumpKind`](probe/type-DumpKind.md) | enum |  | What dump() reports. |
| [`DumpDesc`](probe/type-DumpDesc.md) | desc struct |  | What to dump. |
| [`ArchetypeDump`](probe/type-ArchetypeDump.md) | struct |  | One archetype table. |
| [`SystemDump`](probe/type-SystemDump.md) | struct |  | One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time). |
| [`WorldDump`](probe/type-WorldDump.md) | struct |  | The world: tables, systems, resources. |
| [`EntityDump`](probe/type-EntityDump.md) | struct |  | One entity. |
| [`EntityPrefab`](probe/type-EntityPrefab.md) | struct |  | Where an entity placed from a prefab came from (its og.PrefabLink, resolved) and what its instance overrides. |
| [`DescribedPrefab`](probe/type-DescribedPrefab.md) | struct |  | The instances of one prefab in the world. |
| [`UiElement`](probe/type-UiElement.md) | struct |  | One UI element with its bounds. |
| [`UiSurface`](probe/type-UiSurface.md) | struct |  | The surface UI bounds are laid out on. |
| [`UiDump`](probe/type-UiDump.md) | struct |  | The UI tree, flattened in draw order. |
| [`DesyncDump`](probe/type-DesyncDump.md) | struct |  | The first desync of this peer's lockstep session (og.net), as `dump_desync` writes it. |
| [`DumpResult`](probe/type-DumpResult.md) | union |  | A dump, tagged by its kind. |
| [`ScreenBox`](probe/type-ScreenBox.md) | struct |  | A box on the play surface. |
| [`ScreenPoint`](probe/type-ScreenPoint.md) | struct |  | A point on the play surface. |
| [`DescribedEntity`](probe/type-DescribedEntity.md) | struct |  | One entity the frame shows, from the entity-id pass (a CPU rasterisation of the frame's meshes, skinned meshes and world sprites at a quarter of its pixels). |
| [`DescribedView`](probe/type-DescribedView.md) | struct |  | A secondary camera (og.CameraView, og.Mirror) and what its image shows: its own entity-id pass, from its eye and on its layers, at a quarter of its image's pixels, whatever its `every`. |
| [`DescribedUi`](probe/type-DescribedUi.md) | struct |  | One UI element as describe lists it (the tree dump_ui writes). |
| [`DescribedCamera`](probe/type-DescribedCamera.md) | struct |  | The camera as drawn, with its shake. |
| [`PassSize`](probe/type-PassSize.md) | struct |  | The entity-id pass's size. |
| [`Description`](probe/type-Description.md) | struct |  | The frame as data: deterministic, the same JSON on every host and backend for the same world. |
| [`ExpectStatus`](probe/type-ExpectStatus.md) | enum |  | How an expectation came out. |
| [`ExpectResult`](probe/type-ExpectResult.md) | struct |  | One evaluated expectation. |
| [`ExpectReport`](probe/type-ExpectReport.md) | struct |  | The expectations evaluated since the last call. |
| [`SavedScene`](probe/type-SavedScene.md) | struct |  | The scenes written back from the world: the head is the boot scene's (or the first instantiated one's), `scenes` lists every one. |
| [`SavedSceneFile`](probe/type-SavedSceneFile.md) | struct |  | One scene file written back. |
| [`RuntimeProbe`](probe/type-RuntimeProbe.md) | interface |  | RuntimeProbe protocol 5, member for member. |
| [`EngineProbe`](probe/type-EngineProbe.md) | interface |  | The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring. |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-PROBE_GLOBAL"></a>`PROBE_GLOBAL` | `string` | `"__runtime_probe__"` | The global the page publishes the probe on. |
| <a id="const-PROBE_VERSION"></a>`PROBE_VERSION` | `number` | `5` | The RuntimeProbe contract revision both sides speak. |
| <a id="const-PROBE_WIRE_VERSION"></a>`PROBE_WIRE_VERSION` | `number` | `1` | The wire revision hello reports. |
| <a id="const-REPLAY_SEED_PARAM"></a>`REPLAY_SEED_PARAM` | `string` | `"replay-seed"` | The page query parameter that starts a deterministic replay from a uint32 seed. |
| <a id="const-PLAYTEST_MAX_TICKS"></a>`PLAYTEST_MAX_TICKS` | `number` | `3600` | The longest playtest one request may advance, in fixed steps. |
| <a id="const-PROBE_PROGRAM_MAX_OPERATIONS"></a>`PROBE_PROGRAM_MAX_OPERATIONS` | `number` | `256` | The most operations one probe program may carry. |

```ogt
const PROBE_GLOBAL String "__runtime_probe__" "The global the page publishes the probe on."
```

<sub>`api.ogt` line 23952</sub>

```ogt
const PROBE_VERSION u32 5 "The RuntimeProbe contract revision both sides speak."
```

<sub>`api.ogt` line 23953</sub>

```ogt
const PROBE_WIRE_VERSION u32 1 "The wire revision hello reports."
```

<sub>`api.ogt` line 23954</sub>

```ogt
const REPLAY_SEED_PARAM String "replay-seed" "The page query parameter that starts a deterministic replay from a uint32 seed."
```

<sub>`api.ogt` line 23955</sub>

```ogt
const PLAYTEST_MAX_TICKS u32 3600 "The longest playtest one request may advance, in fixed steps."
```

<sub>`api.ogt` line 23956</sub>

```ogt
const PROBE_PROGRAM_MAX_OPERATIONS u32 256 "The most operations one probe program may carry."
```

<sub>`api.ogt` line 23957</sub>

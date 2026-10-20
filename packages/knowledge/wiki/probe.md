<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `probe` module

The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members.

In a page it is published on globalThis.__runtime_probe__; on native, `og` serves it as JSON-RPC 2.0 over a loopback WebSocket at /probe.

A refusal is false or null exactly as protocol 5; a JSON-RPC error is only for a malformed call or an unknown method.

Probe entity ids are entity names; an unnamed entity is `#<index>.<generation>`.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-build`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module probe {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 10006</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`PROBE_GLOBAL`](#const-PROBE_GLOBAL) | constant | The global the page publishes the probe on. |
| [`PROBE_VERSION`](#const-PROBE_VERSION) | constant | The RuntimeProbe contract revision both sides speak. |
| [`PROBE_WIRE_VERSION`](#const-PROBE_WIRE_VERSION) | constant | The wire revision hello reports. |
| [`REPLAY_SEED_PARAM`](#const-REPLAY_SEED_PARAM) | constant | The page query parameter that starts a deterministic replay from a uint32 seed. |
| [`PLAYTEST_MAX_TICKS`](#const-PLAYTEST_MAX_TICKS) | constant | The longest playtest one request may advance, in fixed steps. |
| [`PROBE_PROGRAM_MAX_OPERATIONS`](#const-PROBE_PROGRAM_MAX_OPERATIONS) | constant | The most operations one probe program may carry. |
| [`PickResult`](#type-PickResult) | struct | What a screen point resolved to. |
| [`ProbeTunable`](#type-ProbeTunable) | struct | One published tunable parameter and the range it means something over. |
| [`ProbeEntityRef`](#type-ProbeEntityRef) | struct | One entity the probe publishes in state(). |
| [`ProbeTimings`](#type-ProbeTimings) | struct | The fixed timings head of state(). |
| [`ProbeState`](#type-ProbeState) | struct | The probe's state(): a fixed head, then the game's own facts. |
| [`AdvanceOperation`](#type-AdvanceOperation) | struct | The advance operation's payload. |
| [`InputOperation`](#type-InputOperation) | struct | The input operation's payload. |
| [`PickOperation`](#type-PickOperation) | struct | The pick operation's payload. |
| [`EntityOperation`](#type-EntityOperation) | struct | The entity operation's payload. |
| [`ProbeOperation`](#type-ProbeOperation) | union | One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip. |
| [`ProbeProgramResult`](#type-ProbeProgramResult) | struct | What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index. |
| [`Hello`](#type-Hello) | struct | The wire's hello: who is speaking. |
| [`BuildProfile`](#type-BuildProfile) | enum | The build configuration; publish builds keep only the player-facing probe members. |
| [`Capture`](#type-Capture) | struct | A captured frame. |
| [`Heard`](#type-Heard) | struct | What the host's mixer rendered last: the audio up to the current tick. |
| [`SweepDesc`](#type-SweepDesc) | desc struct | A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture. |
| [`SweepFrame`](#type-SweepFrame) | struct | One frame of a sweep. |
| [`SweepResult`](#type-SweepResult) | struct | A sweep's frames, one per value. |
| [`ReloadReport`](#type-ReloadReport) | struct | What a hot reload did. Components are re-registered by name and migrated field by field through the schema. |
| [`SnapshotInfo`](#type-SnapshotInfo) | struct | A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process. |
| [`DumpKind`](#type-DumpKind) | enum | What dump() reports. |
| [`DumpDesc`](#type-DumpDesc) | desc struct | What to dump. |
| [`ArchetypeDump`](#type-ArchetypeDump) | struct | One archetype table. |
| [`SystemDump`](#type-SystemDump) | struct | One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time). |
| [`WorldDump`](#type-WorldDump) | struct | The world: tables, systems, resources. |
| [`EntityDump`](#type-EntityDump) | struct | One entity. |
| [`UiElement`](#type-UiElement) | struct | One UI element with its bounds. |
| [`UiSurface`](#type-UiSurface) | struct | The surface UI bounds are laid out on. |
| [`UiDump`](#type-UiDump) | struct | The UI tree, flattened in draw order. |
| [`DumpResult`](#type-DumpResult) | union | A dump, tagged by its kind. |
| [`RuntimeProbe`](#type-RuntimeProbe) | interface | RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values. |
| [`EngineProbe`](#type-EngineProbe) | interface | The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring. |

## Interfaces

<a id="type-RuntimeProbe"></a>

### `RuntimeProbe` (interface)

RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values.

```ogt
interface RuntimeProbe {
  doc "RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values."
  publish false
  # members follow, each quoted below
```

<sub>`api.ogt` line 10303</sub>

<a id="member-RuntimeProbe-version"></a>

#### `RuntimeProbe.version` (property)

PROBE_VERSION.

```ts
readonly version: number
```

Binding symbol `og_runtime_probe_get_version` · publish builds: kept

```ogt
prop version u32 "PROBE_VERSION." {
  publish true
}
```

<sub>`api.ogt` line 10307</sub>

<a id="member-RuntimeProbe-tick"></a>

#### `RuntimeProbe.tick`

Fixed steps the world has taken since load.

```ts
RuntimeProbe.tick(): number
```

**Returns** `number`.

Binding symbol `og_runtime_probe_tick` · publish builds: kept

```ogt
fn tick {
  doc "Fixed steps the world has taken since load."
  publish true
  returns u32
}
```

<sub>`api.ogt` line 10310</sub>

<a id="member-RuntimeProbe-state"></a>

#### `RuntimeProbe.state`

A JSON snapshot: the fixed timings head, then the game's own facts.

```ts
RuntimeProbe.state(): ProbeState
```

**Returns** `ProbeState`.

Binding symbol `og_runtime_probe_state` · publish builds: kept

```ogt
fn state {
  doc "A JSON snapshot: the fixed timings head, then the game's own facts."
  publish true
  returns ProbeState
}
```

<sub>`api.ogt` line 10315</sub>

<a id="member-RuntimeProbe-advance"></a>

#### `RuntimeProbe.advance`

Take whole fixed steps regardless of the wall clock.

```ts
RuntimeProbe.advance(steps: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `steps` | `number` | no | Fixed steps, at most PLAYTEST_MAX_TICKS. |

**Returns** `number`: The tick after the steps.

Binding symbol `og_runtime_probe_advance` · publish builds: dropped

```ogt
fn advance {
  doc "Take whole fixed steps regardless of the wall clock."
  param steps u32 "Fixed steps, at most PLAYTEST_MAX_TICKS."
  returns u32 "The tick after the steps."
}
```

<sub>`api.ogt` line 10320</sub>

<a id="member-RuntimeProbe-pause"></a>

#### `RuntimeProbe.pause`

Hold the wall-clock loop; advance still steps.

```ts
RuntimeProbe.pause(): boolean
```

**Returns** `boolean`.

Binding symbol `og_runtime_probe_pause` · publish builds: dropped

```ogt
fn pause {
  doc "Hold the wall-clock loop; advance still steps."
  returns bool
}
```

<sub>`api.ogt` line 10325</sub>

<a id="member-RuntimeProbe-resume"></a>

#### `RuntimeProbe.resume`

Release the wall-clock loop.

```ts
RuntimeProbe.resume(): boolean
```

**Returns** `boolean`.

Binding symbol `og_runtime_probe_resume` · publish builds: dropped

```ogt
fn resume {
  doc "Release the wall-clock loop."
  returns bool
}
```

<sub>`api.ogt` line 10329</sub>

<a id="member-RuntimeProbe-input"></a>

#### `RuntimeProbe.input`

One event through the input layer's own path.

```ts
RuntimeProbe.input(event: InjectedInput): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `event` | `InjectedInput` | no | The event. |

**Returns** `boolean`: False is a refusal.

Binding symbol `og_runtime_probe_input` · publish builds: kept

```ogt
fn input {
  doc "One event through the input layer's own path."
  publish true
  param event InjectedInput "The event."
  returns bool "False is a refusal."
}
```

<sub>`api.ogt` line 10333</sub>

<a id="member-RuntimeProbe-accepted"></a>

#### `RuntimeProbe.accepted`

Page events (keyboard, pointer, wheel) the input layer accepted since load.

```ts
RuntimeProbe.accepted(): number
```

**Returns** `number`.

Binding symbol `og_runtime_probe_accepted` · publish builds: dropped

```ogt
fn accepted {
  doc "Page events (keyboard, pointer, wheel) the input layer accepted since load."
  returns u32
}
```

<sub>`api.ogt` line 10339</sub>

<a id="member-RuntimeProbe-pick"></a>

#### `RuntimeProbe.pick`

What is under a point of the play surface.

```ts
RuntimeProbe.pick(x: number, y: number): PickResult | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | Horizontal position in CSS pixels. |
| `y` | `number` | no | Vertical position in CSS pixels. |

**Returns** `PickResult | null`: Null when nothing is under the point.

Binding symbol `og_runtime_probe_pick` · publish builds: kept

```ogt
fn pick {
  doc "What is under a point of the play surface."
  publish true
  param x f32 "Horizontal position in CSS pixels."
  param y f32 "Vertical position in CSS pixels."
  returns Option.PickResult "Null when nothing is under the point."
}
```

<sub>`api.ogt` line 10343</sub>

<a id="member-RuntimeProbe-entity"></a>

#### `RuntimeProbe.entity`

One entity's published parameters: its tunable fields as `Component.field` keys.

```ts
RuntimeProbe.entity(id: string): Readonly<Record<string, unknown>> | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The probe id. |

**Returns** `Readonly<Record<string, unknown>> | null`: Null for an unknown entity.

Binding symbol `og_runtime_probe_entity` · publish builds: kept

```ogt
fn entity {
  doc "One entity's published parameters: its tunable fields as `Component.field` keys."
  publish true
  param id String "The probe id."
  returns Option.Map.Json "Null for an unknown entity."
}
```

<sub>`api.ogt` line 10350</sub>

<a id="member-RuntimeProbe-bounds"></a>

#### `RuntimeProbe.bounds`

The ranges of one entity's tunable fields, from the schema.

```ts
RuntimeProbe.bounds(id: string): Readonly<Record<string, ProbeTunable>> | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The probe id. |

**Returns** `Readonly<Record<string, ProbeTunable>> | null`: Null for an unknown entity.

Binding symbol `og_runtime_probe_bounds` · publish builds: dropped

```ogt
fn bounds {
  doc "The ranges of one entity's tunable fields, from the schema."
  param id String "The probe id."
  returns Option.Map.ProbeTunable "Null for an unknown entity."
}
```

<sub>`api.ogt` line 10356</sub>

<a id="member-RuntimeProbe-patch"></a>

#### `RuntimeProbe.patch`

Move one tunable field for this run only; it applies within one frame. The probe never writes project files.

```ts
RuntimeProbe.patch(id: string, parameter: string, value: unknown): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The probe id. |
| `parameter` | `string` | no | The field, as `Health.hp`. |
| `value` | `unknown` | no | The new value. |

**Returns** `boolean`: False is a refusal.

Binding symbol `og_runtime_probe_patch` · publish builds: dropped

```ogt
fn patch {
  doc "Move one tunable field for this run only; it applies within one frame. The probe never writes project files."
  param id String "The probe id."
  param parameter String "The field, as `Health.hp`."
  param value Json "The new value."
  returns bool "False is a refusal."
}
```

<sub>`api.ogt` line 10361</sub>

<a id="member-RuntimeProbe-run"></a>

#### `RuntimeProbe.run`

Run a program of the members above in order, stopping at the first refusal.

```ts
RuntimeProbe.run(program: readonly ProbeOperation[]): ProbeProgramResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `program` | `readonly ProbeOperation[]` | no | At most PROBE_PROGRAM_MAX_OPERATIONS operations. |

**Returns** `ProbeProgramResult`.

Binding symbol `og_runtime_probe_run` · publish builds: dropped

```ogt
fn run {
  doc "Run a program of the members above in order, stopping at the first refusal."
  param program List.ProbeOperation "At most PROBE_PROGRAM_MAX_OPERATIONS operations."
  returns ProbeProgramResult
}
```

<sub>`api.ogt` line 10368</sub>

<a id="type-EngineProbe"></a>

### `EngineProbe` (interface)

The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring.

Extends [`RuntimeProbe`](#type-RuntimeProbe): every member of it is a member here.

```ogt
interface EngineProbe {
  doc "The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring."
  extends RuntimeProbe
  publish false
  # members follow, each quoted below
```

<sub>`api.ogt` line 10375</sub>

<a id="member-EngineProbe-hello"></a>

#### `EngineProbe.hello`

Who is speaking: protocol, wire, engine version, tick rate, renderer tier, build.

```ts
EngineProbe.hello(): Hello
```

**Returns** `Hello`.

Binding symbol `og_engine_probe_hello` · publish builds: kept

```ogt
fn hello {
  doc "Who is speaking: protocol, wire, engine version, tick rate, renderer tier, build."
  publish true
  returns Hello
}
```

<sub>`api.ogt` line 10380</sub>

<a id="member-EngineProbe-capture"></a>

#### `EngineProbe.capture`

The frame of the current tick, rendered offscreen and read back as a PNG.

Size rule: width and height are both 0 (or absent), which is the render target's size, or both 1 to 4096, which renders that size with the same camera (the aspect follows the requested size); one zero and one not is a malformed call.

On the web the readback is asynchronous: through the request ring the answer arrives when the copy completes; a synchronous in-page call answers null on the first tick and the frame from then on (a capture is requested on tick N and answered on N + 1).

```ts
EngineProbe.capture(width?: number, height?: number): Capture | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `width` | `number` | yes | Width in pixels; 0 is the render target's. Default `0`. |
| `height` | `number` | yes | Height in pixels; 0 is the render target's. Default `0`. |

**Returns** `Capture | null`: Null without a renderer.

Binding symbol `og_engine_probe_capture` · publish builds: kept

```ogt
fn capture {
  doc "The frame of the current tick, rendered offscreen and read back as a PNG."
  doc "Size rule: width and height are both 0 (or absent), which is the render target's size, or both 1 to 4096, which renders that size with the same camera (the aspect follows the requested size); one zero and one not is a malformed call."
  doc "On the web the readback is asynchronous: through the request ring the answer arrives when the copy completes; a synchronous in-page call answers null on the first tick and the frame from then on (a capture is requested on tick N and answered on N + 1)."
  publish true
  param width u32 "Width in pixels; 0 is the render target's." {
    default 0
  }
  param height u32 "Height in pixels; 0 is the render target's." {
    default 0
  }
  returns Option.Capture "Null without a renderer."
}
```

<sub>`api.ogt` line 10385</sub>

<a id="member-EngineProbe-sweep"></a>

#### `EngineProbe.sweep`

A filmstrip over one tunable: per value, restore, patch, advance, capture.

```ts
EngineProbe.sweep(desc?: SweepDesc): SweepResult | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SweepDesc` | yes | The parameter, values, steps and size. Its fields travel as the call's own parameters on the wire. |

**Returns** `SweepResult | null`: Null for an unknown entity or parameter, or without a renderer.

Binding symbol `og_engine_probe_sweep` · publish builds: dropped

```ogt
fn sweep {
  doc "A filmstrip over one tunable: per value, restore, patch, advance, capture."
  param desc SweepDesc "The parameter, values, steps and size." {
    flatten true
  }
  returns Option.SweepResult "Null for an unknown entity or parameter, or without a renderer."
}
```

<sub>`api.ogt` line 10398</sub>

<a id="member-EngineProbe-reload"></a>

#### `EngineProbe.reload`

Hot reload: snapshot, swap the game bundle, re-register components by name, restore.

```ts
EngineProbe.reload(): ReloadReport | null
```

**Returns** `ReloadReport | null`: Null when the new bundle fails to load; the old one keeps running.

Binding symbol `og_engine_probe_reload` · publish builds: dropped

```ogt
fn reload {
  doc "Hot reload: snapshot, swap the game bundle, re-register components by name, restore."
  returns Option.ReloadReport "Null when the new bundle fails to load; the old one keeps running."
}
```

<sub>`api.ogt` line 10405</sub>

<a id="member-EngineProbe-snapshot"></a>

#### `EngineProbe.snapshot`

Snapshot the whole world in the running process.

```ts
EngineProbe.snapshot(): SnapshotInfo | null
```

**Returns** `SnapshotInfo | null`.

Binding symbol `og_engine_probe_snapshot` · publish builds: dropped

```ogt
fn snapshot {
  doc "Snapshot the whole world in the running process."
  returns Option.SnapshotInfo
}
```

<sub>`api.ogt` line 10409</sub>

<a id="member-EngineProbe-restore"></a>

#### `EngineProbe.restore`

Restore a snapshot taken by this process.

```ts
EngineProbe.restore(id: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The snapshot id. |

**Returns** `boolean`: False for an unknown id.

Binding symbol `og_engine_probe_restore` · publish builds: dropped

```ogt
fn restore {
  doc "Restore a snapshot taken by this process."
  param id String "The snapshot id."
  returns bool "False for an unknown id."
}
```

<sub>`api.ogt` line 10413</sub>

<a id="member-EngineProbe-dump"></a>

#### `EngineProbe.dump`

A world, entity or UI dump, as the command channel's dump actions write.

```ts
EngineProbe.dump(desc?: DumpDesc): DumpResult | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `DumpDesc` | yes | What to dump. Its fields travel as the call's own parameters on the wire. |

**Returns** `DumpResult | null`: Null for an unknown entity.

Binding symbol `og_engine_probe_dump` · publish builds: dropped

```ogt
fn dump {
  doc "A world, entity or UI dump, as the command channel's dump actions write."
  param desc DumpDesc "What to dump." {
    flatten true
  }
  returns Option.DumpResult "Null for an unknown entity."
}
```

<sub>`api.ogt` line 10418</sub>

<a id="member-EngineProbe-commands"></a>

#### `EngineProbe.commands`

Schedule a command script on simulation time, as `og run --commands`.

```ts
EngineProbe.commands(script: string): CommandsReport | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `script` | `string` | no | `t:action[:args];...`. |

**Returns** `CommandsReport | null`: Null for a malformed script.

Binding symbol `og_engine_probe_commands` · publish builds: dropped

```ogt
fn commands {
  doc "Schedule a command script on simulation time, as `og run --commands`."
  param script String "`t:action[:args];...`."
  returns Option.CommandsReport "Null for a malformed script."
}
```

<sub>`api.ogt` line 10425</sub>

<a id="member-EngineProbe-listen"></a>

#### `EngineProbe.listen`

The last seconds of audio the host's mixer rendered (a null sink renders too, into a two-second ring), as a WAV with its peak and RMS: what a person would have heard.

```ts
EngineProbe.listen(seconds?: number): Heard | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `seconds` | `number` | yes | How many seconds, 0 to 2. Default `1`. |

**Returns** `Heard | null`: Null without audio, or before anything was rendered.

Binding symbol `og_engine_probe_listen` · publish builds: dropped

```ogt
fn listen {
  doc "The last seconds of audio the host's mixer rendered (a null sink renders too, into a two-second ring), as a WAV with its peak and RMS: what a person would have heard."
  param seconds f32 "How many seconds, 0 to 2." {
    default 1
  }
  returns Option.Heard "Null without audio, or before anything was rendered."
}
```

<sub>`api.ogt` line 10430</sub>

## Types

<a id="type-PickResult"></a>

### `PickResult` (struct)

What a screen point resolved to.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `string` | `""` | — | — | — | The probe id of the entity under the point. |
| `world` | `[number, number] \| [number, number, number]` | — | — | — | — | Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero. |
| `source` | `string \| null` | `null` | — | — | — | Where a person would change it, as `player.ts::jump`. |

```ogt
struct PickResult {
  doc "What a screen point resolved to."
  field entity String "The probe id of the entity under the point."
  field world OneOf.Array.f32.2.Array.f32.3 "Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero."
  field source Option.String "Where a person would change it, as `player.ts::jump`."
}
```

<sub>`api.ogt` line 10020</sub>

<a id="type-ProbeTunable"></a>

### `ProbeTunable` (struct)

One published tunable parameter and the range it means something over.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Where the value lives, as `Health.hp`. |
| `label` | `string` | `""` | — | — | optional | The person-facing name. |
| `min` | `number` | `0` | — | — | optional | The smallest meaningful value. |
| `max` | `number` | `0` | — | — | optional | The largest meaningful value. |
| `step` | `number` | `0` | — | — | optional | Tune's increment. |

```ogt
struct ProbeTunable {
  doc "One published tunable parameter and the range it means something over."
  field path String "Where the value lives, as `Health.hp`."
  field label String "The person-facing name." {
    optional true
  }
  field min f32 "The smallest meaningful value." {
    optional true
  }
  field max f32 "The largest meaningful value." {
    optional true
  }
  field step f32 "Tune's increment." {
    optional true
  }
}
```

<sub>`api.ogt` line 10027</sub>

<a id="type-ProbeEntityRef"></a>

### `ProbeEntityRef` (struct)

One entity the probe publishes in state().

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `label` | `string` | `""` | — | — | optional | A person-facing name. |

```ogt
struct ProbeEntityRef {
  doc "One entity the probe publishes in state()."
  field id String "Its probe id."
  field label String "A person-facing name." {
    optional true
  }
}
```

<sub>`api.ogt` line 10044</sub>

<a id="type-ProbeTimings"></a>

### `ProbeTimings` (struct)

The fixed timings head of state().

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `number` | `0` | — | `ms` | — | Time the last tick took. |
| `render` | `number` | `0` | — | `ms` | — | Time the last frame's render took; 0 without a renderer. |
| `frameMs` | `number` | `0` | — | `ms` | — | step + render. |

```ogt
struct ProbeTimings {
  doc "The fixed timings head of state()."
  field step f32 "Time the last tick took." {
    unit ms
  }
  field render f32 "Time the last frame's render took; 0 without a renderer." {
    unit ms
  }
  field frameMs f32 "step + render." {
    unit ms
  }
}
```

<sub>`api.ogt` line 10052</sub>

<a id="type-ProbeState"></a>

### `ProbeState` (struct)

The probe's state(): a fixed head, then the game's own facts.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | Fixed steps since load. |
| `paused` | `boolean` | `false` | — | — | — | Whether the clock is held. |
| `seed` | `number` | `0` | — | — | — | The seed the world was assembled from. |
| `timings` | `ProbeTimings` | `{ step: 0, render: 0, frameMs: 0 }` | — | — | — | The last tick's timings. |
| `entities` | `readonly ProbeEntityRef[]` | `[]` | — | — | — | The named entities. |
| `facts` | `unknown` | `null` | — | — | — | What GameDesc.facts returned, or null. |

```ogt
struct ProbeState {
  doc "The probe's state(): a fixed head, then the game's own facts."
  field tick u32 "Fixed steps since load."
  field paused bool "Whether the clock is held."
  field seed u32 "The seed the world was assembled from."
  field timings ProbeTimings "The last tick's timings."
  field entities List.ProbeEntityRef "The named entities."
  field facts Json "What GameDesc.facts returned, or null."
}
```

<sub>`api.ogt` line 10065</sub>

<a id="type-AdvanceOperation"></a>

### `AdvanceOperation` (struct)

The advance operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `steps` | `number` | `0` | — | — | — | Fixed steps to take. |

```ogt
struct AdvanceOperation {
  doc "The advance operation's payload."
  field steps u32 "Fixed steps to take."
}
```

<sub>`api.ogt` line 10075</sub>

<a id="type-InputOperation"></a>

### `InputOperation` (struct)

The input operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `event` | `InjectedInput` | — | — | — | — | The event. |

```ogt
struct InputOperation {
  doc "The input operation's payload."
  field event InjectedInput "The event."
}
```

<sub>`api.ogt` line 10080</sub>

<a id="type-PickOperation"></a>

### `PickOperation` (struct)

The pick operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |

```ogt
struct PickOperation {
  doc "The pick operation's payload."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
}
```

<sub>`api.ogt` line 10085</sub>

<a id="type-EntityOperation"></a>

### `EntityOperation` (struct)

The entity operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The probe id. |

```ogt
struct EntityOperation {
  doc "The entity operation's payload."
  field id String "The probe id."
}
```

<sub>`api.ogt` line 10091</sub>

<a id="type-ProbeOperation"></a>

### `ProbeOperation` (union)

One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip.

Tagged by `op`.

| Case | Payload | Doc |
|---|---|---|
| `op: "advance"` | [`AdvanceOperation`](#type-AdvanceOperation) | Take fixed steps; answers the tick. |
| `op: "input"` | [`InputOperation`](#type-InputOperation) | Inject an event; answers true. |
| `op: "pause"` | — | Hold the clock; answers true. |
| `op: "resume"` | — | Release the clock; answers true. |
| `op: "read"` | — | Read the state; answers `{ tick, state }`. |
| `op: "pick"` | [`PickOperation`](#type-PickOperation) | Pick a point; answers its result or null. |
| `op: "entity"` | [`EntityOperation`](#type-EntityOperation) | Read an entity's parameters; null is a refusal. |

```ogt
union ProbeOperation {
  doc "One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip."
  tag op
  case advance AdvanceOperation "Take fixed steps; answers the tick."
  case input InputOperation "Inject an event; answers true."
  case pause "Hold the clock; answers true."
  case resume "Release the clock; answers true."
  case read "Read the state; answers `{ tick, state }`."
  case pick PickOperation "Pick a point; answers its result or null."
  case entity EntityOperation "Read an entity's parameters; null is a refusal."
}
```

<sub>`api.ogt` line 10096</sub>

<a id="type-ProbeProgramResult"></a>

### `ProbeProgramResult` (struct)

What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `results` | `readonly unknown[]` | `[]` | — | — | — | The answers, in order. |
| `refused` | `number \| null` | `null` | — | — | — | The index of the refusing operation, or null. |
| `tick` | `number` | `0` | — | — | — | The tick after the last operation that ran. |

```ogt
struct ProbeProgramResult {
  doc "What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index."
  field results List.Json "The answers, in order."
  field refused Option.u32 "The index of the refusing operation, or null."
  field tick u32 "The tick after the last operation that ran."
}
```

<sub>`api.ogt` line 10108</sub>

<a id="type-Hello"></a>

### `Hello` (struct)

The wire's hello: who is speaking.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `protocol` | `number` | `0` | — | — | — | PROBE_VERSION. |
| `wire` | `number` | `0` | — | — | — | PROBE_WIRE_VERSION. |
| `substrate` | `string` | `""` | — | — | — | Always `origin-engine`. |
| `engine` | `string` | `""` | — | — | — | `origin-engine <version>`. |
| `ticksPerSecond` | `number` | `0` | — | `hz` | — | The fixed tick rate. |
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The renderer backend. |
| `build` | `BuildProfile` | `"dev"` | — | — | — | The build configuration. |
| `members` | `readonly string[]` | `[]` | — | — | — | The probe members this build serves. |
| `modules` | `readonly string[]` | `[]` | — | — | — | The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something. |

```ogt
struct Hello {
  doc "The wire's hello: who is speaking."
  field protocol u32 "PROBE_VERSION."
  field wire u32 "PROBE_WIRE_VERSION."
  field substrate String "Always `origin-engine`."
  field engine String "`origin-engine <version>`."
  field ticksPerSecond u32 "The fixed tick rate." {
    unit hz
  }
  field tier RenderTier "The renderer backend."
  field build BuildProfile "The build configuration."
  field members List.String "The probe members this build serves."
  field modules List.String "The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something."
}
```

<sub>`api.ogt` line 10115</sub>

<a id="type-BuildProfile"></a>

### `BuildProfile` (enum)

The build configuration; publish builds keep only the player-facing probe members.

A string: one of the variants below. The zero value is `"dev"`.

| Variant | Doc |
|---|---|
| `"dev"` | A development build: every member, the command channel, dumps. |
| `"release"` | An optimised build with every member. |
| `"publish"` | A published game: hello, tick, state, pick, entity, input and capture only. |

```ogt
enum BuildProfile {
  doc "The build configuration; publish builds keep only the player-facing probe members."
  variant dev "A development build: every member, the command channel, dumps."
  variant release "An optimised build with every member."
  variant publish "A published game: hello, tick, state, pick, entity, input and capture only."
}
```

<sub>`api.ogt` line 10130</sub>

<a id="type-Capture"></a>

### `Capture` (struct)

A captured frame.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the frame shows. |
| `width` | `number` | `0` | — | — | — | Width in pixels. |
| `height` | `number` | `0` | — | — | — | Height in pixels. |
| `png` | `string` | `""` | — | — | — | The PNG, base64. |

```ogt
struct Capture {
  doc "A captured frame."
  field tick u32 "The tick the frame shows."
  field width u32 "Width in pixels."
  field height u32 "Height in pixels."
  field png String "The PNG, base64."
}
```

<sub>`api.ogt` line 10137</sub>

<a id="type-Heard"></a>

### `Heard` (struct)

What the host's mixer rendered last: the audio up to the current tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick it ends at. |
| `rate` | `number` | `0` | — | `hz` | — | Frames per second. |
| `frames` | `number` | `0` | — | — | — | Stereo frames in the WAV. |
| `peak` | `number` | `0` | — | — | — | The largest sample, 0 to 1. |
| `rms` | `number` | `0` | — | — | — | The root mean square of both channels. |
| `wav` | `string` | `""` | — | — | — | A 16-bit stereo WAV, base64. |

```ogt
struct Heard {
  doc "What the host's mixer rendered last: the audio up to the current tick."
  field tick u32 "The tick it ends at."
  field rate u32 "Frames per second." {
    unit hz
  }
  field frames u32 "Stereo frames in the WAV."
  field peak f32 "The largest sample, 0 to 1."
  field rms f32 "The root mean square of both channels."
  field wav String "A 16-bit stereo WAV, base64."
}
```

<sub>`api.ogt` line 10145</sub>

<a id="type-SweepDesc"></a>

### `SweepDesc` (desc struct)

A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The probe id of the entity. |
| `parameter` | `string` | `""` | — | — | — | The parameter, as `Health.hp`. |
| `values` | `readonly unknown[]` | `[]` | — | — | — | The values to try, in order. |
| `steps` | `number` | `0` | — | — | — | Fixed steps to advance after each patch. |
| `width` | `number` | `0` | — | — | — | Capture width; 0 is the render target's. |
| `height` | `number` | `0` | — | — | — | Capture height; 0 is the render target's. |

```ogt
struct SweepDesc {
  doc "A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture."
  desc true
  field id String "The probe id of the entity."
  field parameter String "The parameter, as `Health.hp`."
  field values List.Json "The values to try, in order."
  field steps u32 "Fixed steps to advance after each patch."
  field width u32 "Capture width; 0 is the render target's."
  field height u32 "Capture height; 0 is the render target's."
}
```

<sub>`api.ogt` line 10157</sub>

<a id="type-SweepFrame"></a>

### `SweepFrame` (struct)

One frame of a sweep.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `unknown` | `null` | — | — | — | The value patched in. |
| `tick` | `number` | `0` | — | — | — | The tick captured. |
| `png` | `string` | `""` | — | — | — | The PNG, base64. |

```ogt
struct SweepFrame {
  doc "One frame of a sweep."
  field value Json "The value patched in."
  field tick u32 "The tick captured."
  field png String "The PNG, base64."
}
```

<sub>`api.ogt` line 10168</sub>

<a id="type-SweepResult"></a>

### `SweepResult` (struct)

A sweep's frames, one per value.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `frames` | `readonly SweepFrame[]` | `[]` | — | — | — | The frames. |

```ogt
struct SweepResult {
  doc "A sweep's frames, one per value."
  field frames List.SweepFrame "The frames."
}
```

<sub>`api.ogt` line 10175</sub>

<a id="type-ReloadReport"></a>

### `ReloadReport` (struct)

What a hot reload did. Components are re-registered by name and migrated field by field through the schema.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the world was restored at. |
| `ms` | `number` | `0` | — | `ms` | — | From bundle swap to restored world. |
| `migrated` | `readonly string[]` | `[]` | — | — | — | Fields carried across a layout change, as `Health.hp`. |
| `dropped` | `readonly string[]` | `[]` | — | — | — | Components or fields that no longer exist, dropped with their data. |
| `warnings` | `readonly string[]` | `[]` | — | — | — | Anything else worth reading, each naming the fix. |

```ogt
struct ReloadReport {
  doc "What a hot reload did. Components are re-registered by name and migrated field by field through the schema."
  field tick u32 "The tick the world was restored at."
  field ms f32 "From bundle swap to restored world." {
    unit ms
  }
  field migrated List.String "Fields carried across a layout change, as `Health.hp`."
  field dropped List.String "Components or fields that no longer exist, dropped with their data."
  field warnings List.String "Anything else worth reading, each naming the fix."
}
```

<sub>`api.ogt` line 10180</sub>

<a id="type-SnapshotInfo"></a>

### `SnapshotInfo` (struct)

A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The snapshot's id, for restore. |
| `tick` | `number` | `0` | — | — | — | The tick it was taken at. |
| `hash` | `string` | `""` | — | — | — | The world hash at that tick, hex: equal hashes mean equal worlds. |
| `bytes` | `number` | `0` | — | `bytes` | — | Its size. |

```ogt
struct SnapshotInfo {
  doc "A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process."
  field id String "The snapshot's id, for restore."
  field tick u32 "The tick it was taken at."
  field hash String "The world hash at that tick, hex: equal hashes mean equal worlds."
  field bytes u32 "Its size." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 10191</sub>

<a id="type-DumpKind"></a>

### `DumpKind` (enum)

What dump() reports.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | Archetype tables, systems and their matched counts. |
| `"entity"` | One entity's components and transform. |
| `"ui"` | The UI tree with bounds. |

```ogt
enum DumpKind {
  doc "What dump() reports."
  variant world "Archetype tables, systems and their matched counts."
  variant entity "One entity's components and transform."
  variant ui "The UI tree with bounds."
}
```

<sub>`api.ogt` line 10201</sub>

<a id="type-DumpDesc"></a>

### `DumpDesc` (desc struct)

What to dump.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `what` | `DumpKind` | `"world"` | — | — | — | The kind of dump. |
| `id` | `string` | `""` | — | — | — | The probe id, for an entity dump. |

```ogt
struct DumpDesc {
  doc "What to dump."
  desc true
  field what DumpKind "The kind of dump."
  field id String "The probe id, for an entity dump."
}
```

<sub>`api.ogt` line 10208</sub>

<a id="type-ArchetypeDump"></a>

### `ArchetypeDump` (struct)

One archetype table.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `components` | `readonly string[]` | `[]` | — | — | — | Its components, by name. |
| `count` | `number` | `0` | — | — | — | Entities in it. |

```ogt
struct ArchetypeDump {
  doc "One archetype table."
  field components List.String "Its components, by name."
  field count u32 "Entities in it."
}
```

<sub>`api.ogt` line 10215</sub>

<a id="type-SystemDump"></a>

### `SystemDump` (struct)

One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `matched` | `number \| null` | `null` | — | — | — | Entities its queries match now; null for a system the host did not declare to the probe. |
| `reads` | `readonly string[] \| null` | `null` | — | — | — | What it declared it reads; null when undeclared. |
| `writes` | `readonly string[] \| null` | `null` | — | — | — | What it declared it writes; null when undeclared. |
| `ms` | `number \| null` | `null` | — | `ms` | — | Time it took on the last tick; null when not measured. |

```ogt
struct SystemDump {
  doc "One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time)."
  field name String "Its name."
  field matched Option.u32 "Entities its queries match now; null for a system the host did not declare to the probe."
  field reads Option.List.String "What it declared it reads; null when undeclared."
  field writes Option.List.String "What it declared it writes; null when undeclared."
  field ms Option.f32 "Time it took on the last tick; null when not measured." {
    unit ms
  }
}
```

<sub>`api.ogt` line 10221</sub>

<a id="type-WorldDump"></a>

### `WorldDump` (struct)

The world: tables, systems, resources.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `hash` | `string` | `""` | — | — | — | The world hash, hex. |
| `entities` | `number` | `0` | — | — | — | Live entities. |
| `archetypes` | `readonly ArchetypeDump[]` | `[]` | — | — | — | The archetype tables. |
| `systems` | `readonly SystemDump[]` | `[]` | — | — | — | The systems in run order. |
| `resources` | `readonly string[]` | `[]` | — | — | — | The registered resources. |

```ogt
struct WorldDump {
  doc "The world: tables, systems, resources."
  field tick u32 "The tick."
  field hash String "The world hash, hex."
  field entities u32 "Live entities."
  field archetypes List.ArchetypeDump "The archetype tables."
  field systems List.SystemDump "The systems in run order."
  field resources List.String "The registered resources."
}
```

<sub>`api.ogt` line 10232</sub>

<a id="type-EntityDump"></a>

### `EntityDump` (struct)

One entity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `name` | `string \| null` | `null` | — | — | — | Its name. |
| `components` | `Readonly<Record<string, unknown>>` | `{}` | — | — | — | Each component's field values, by component name. |

```ogt
struct EntityDump {
  doc "One entity."
  field id String "Its probe id."
  field name Option.String "Its name."
  field components Map.Json "Each component's field values, by component name."
}
```

<sub>`api.ogt` line 10242</sub>

<a id="type-UiElement"></a>

### `UiElement` (struct)

One UI element with its bounds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its stable id, as the command channel targets it. |
| `parent` | `string` | `""` | — | — | — | Its parent's id; empty at the root. |
| `kind` | `string` | `""` | — | — | — | What made it: `label`, `button`, `icon`, `image`, `panel` and the other widget names; empty when the UI module does not say. |
| `disabled` | `boolean` | `false` | — | — | — | A disabled control (its text is exempt from contrast rules, as WCAG exempts inactive controls). |
| `clipped` | `boolean` | `false` | — | — | — | Only part of it shows: a scroll container or the surface cuts it, and the bounds are the part that shows. |
| `text` | `string` | `""` | — | — | — | The text it shows, if any. |
| `role` | `string` | `""` | — | — | — | Its accessibility role (og.ui.a11y): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, alert, dialog, tooltip, separator, group; empty for a presentational element (a layout container, a spacer). |
| `name` | `string` | `""` | — | — | — | Its accessible name: its text, a name the widget gives it (an icon's name, a dialog's title, Decrease and Increase for a stepper's buttons), a control's own text gathered from its children, or else its id. |
| `value` | `string` | `""` | — | — | — | Its value as text: a slider's or stepper's number, a progress's or meter's amount, a field's text, a select's choice; empty for none. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states: disabled, focused, focusable (keys and pads reach it now, in this order), checked, selected, expanded, pressed, busy. |
| `x` | `number` | `0` | — | `cssPx` | — | Left edge. |
| `y` | `number` | `0` | — | `cssPx` | — | Top edge. |
| `width` | `number` | `0` | — | `cssPx` | — | Width. |
| `height` | `number` | `0` | — | `cssPx` | — | Height. |

```ogt
struct UiElement {
  doc "One UI element with its bounds."
  field id String "Its stable id, as the command channel targets it."
  field parent String "Its parent's id; empty at the root."
  field kind String "What made it: `label`, `button`, `icon`, `image`, `panel` and the other widget names; empty when the UI module does not say."
  field disabled bool "A disabled control (its text is exempt from contrast rules, as WCAG exempts inactive controls)."
  field clipped bool "Only part of it shows: a scroll container or the surface cuts it, and the bounds are the part that shows."
  field text String "The text it shows, if any."
  field role String "Its accessibility role (og.ui.a11y): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, alert, dialog, tooltip, separator, group; empty for a presentational element (a layout container, a spacer)."
  field name String "Its accessible name: its text, a name the widget gives it (an icon's name, a dialog's title, Decrease and Increase for a stepper's buttons), a control's own text gathered from its children, or else its id."
  field value String "Its value as text: a slider's or stepper's number, a progress's or meter's amount, a field's text, a select's choice; empty for none."
  field states List.String "Its states: disabled, focused, focusable (keys and pads reach it now, in this order), checked, selected, expanded, pressed, busy."
  field x f32 "Left edge." {
    unit cssPx
  }
  field y f32 "Top edge." {
    unit cssPx
  }
  field width f32 "Width." {
    unit cssPx
  }
  field height f32 "Height." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 10249</sub>

<a id="type-UiSurface"></a>

### `UiSurface` (struct)

The surface UI bounds are laid out on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |
| `dpr` | `number` | `1` | — | — | — | Device pixels per CSS pixel: a frame of this surface is width × dpr pixels wide. |

```ogt
struct UiSurface {
  doc "The surface UI bounds are laid out on."
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
  field dpr f32 "Device pixels per CSS pixel: a frame of this surface is width × dpr pixels wide." {
    default 1
  }
}
```

<sub>`api.ogt` line 10275</sub>

<a id="type-UiDump"></a>

### `UiDump` (struct)

The UI tree, flattened in draw order.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `surface` | `UiSurface \| null` | `null` | — | — | — | The surface the bounds are on; null without a UI module. |
| `elements` | `readonly UiElement[]` | `[]` | — | — | — | The elements. |

```ogt
struct UiDump {
  doc "The UI tree, flattened in draw order."
  field tick u32 "The tick."
  field surface Option.UiSurface "The surface the bounds are on; null without a UI module."
  field elements List.UiElement "The elements."
}
```

<sub>`api.ogt` line 10288</sub>

<a id="type-DumpResult"></a>

### `DumpResult` (union)

A dump, tagged by its kind.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "world"` | [`WorldDump`](#type-WorldDump) | The world dump. |
| `kind: "entity"` | [`EntityDump`](#type-EntityDump) | An entity dump. |
| `kind: "ui"` | [`UiDump`](#type-UiDump) | The UI dump. |

```ogt
union DumpResult {
  doc "A dump, tagged by its kind."
  tag kind
  case world WorldDump "The world dump."
  case entity EntityDump "An entity dump."
  case ui UiDump "The UI dump."
}
```

<sub>`api.ogt` line 10295</sub>

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

<sub>`api.ogt` line 10013</sub>

```ogt
const PROBE_VERSION u32 5 "The RuntimeProbe contract revision both sides speak."
```

<sub>`api.ogt` line 10014</sub>

```ogt
const PROBE_WIRE_VERSION u32 1 "The wire revision hello reports."
```

<sub>`api.ogt` line 10015</sub>

```ogt
const REPLAY_SEED_PARAM String "replay-seed" "The page query parameter that starts a deterministic replay from a uint32 seed."
```

<sub>`api.ogt` line 10016</sub>

```ogt
const PLAYTEST_MAX_TICKS u32 3600 "The longest playtest one request may advance, in fixed steps."
```

<sub>`api.ogt` line 10017</sub>

```ogt
const PROBE_PROGRAM_MAX_OPERATIONS u32 256 "The most operations one probe program may carry."
```

<sub>`api.ogt` line 10018</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-RuntimeProbe"></a>

### `RuntimeProbe` (interface)

RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values.

```ogt
interface RuntimeProbe {
  doc "RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values."
  publish false
  # members follow, each quoted below
```

<sub>`api.ogt` line 24621</sub>

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

<sub>`api.ogt` line 24625</sub>

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

<sub>`api.ogt` line 24628</sub>

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

<sub>`api.ogt` line 24633</sub>

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

<sub>`api.ogt` line 24638</sub>

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

<sub>`api.ogt` line 24643</sub>

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

<sub>`api.ogt` line 24647</sub>

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

<sub>`api.ogt` line 24651</sub>

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

<sub>`api.ogt` line 24657</sub>

<a id="member-RuntimeProbe-pick"></a>

#### `RuntimeProbe.pick`

The scene entity under a point of the play surface, and the UI element over it (pickUi's answer) outside a publish build.

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
  doc "The scene entity under a point of the play surface, and the UI element over it (pickUi's answer) outside a publish build."
  publish true
  param x f32 "Horizontal position in CSS pixels."
  param y f32 "Vertical position in CSS pixels."
  returns Option.PickResult "Null when nothing is under the point."
}
```

<sub>`api.ogt` line 24661</sub>

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

<sub>`api.ogt` line 24668</sub>

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

<sub>`api.ogt` line 24674</sub>

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

<sub>`api.ogt` line 24679</sub>

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

<sub>`api.ogt` line 24686</sub>

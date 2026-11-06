<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-EngineProbe"></a>

### `EngineProbe` (interface)

The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring.

Extends [`RuntimeProbe`](../probe/type-RuntimeProbe.md#type-RuntimeProbe): every member of it is a member here.

```ogt
interface EngineProbe {
  doc "The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring."
  extends RuntimeProbe
  publish false
  # members follow, each quoted below
```

<sub>`api.ogt` line 24569</sub>

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

<sub>`api.ogt` line 24574</sub>

<a id="member-EngineProbe-capture"></a>

#### `EngineProbe.capture`

The frame of the current tick, rendered offscreen and read back as a PNG; null without a renderer (hello's tier `none`).

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
  doc "The frame of the current tick, rendered offscreen and read back as a PNG; null without a renderer (hello's tier `none`)."
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

<sub>`api.ogt` line 24579</sub>

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

<sub>`api.ogt` line 24592</sub>

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

<sub>`api.ogt` line 24599</sub>

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

<sub>`api.ogt` line 24603</sub>

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

<sub>`api.ogt` line 24607</sub>

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

<sub>`api.ogt` line 24612</sub>

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

<sub>`api.ogt` line 24619</sub>

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

<sub>`api.ogt` line 24624</sub>

<a id="member-EngineProbe-memory"></a>

#### `EngineProbe.memory`

What the engine holds: the wasm linear memory on the web or the process's resident memory natively, the engine's counted heap, the world's entities, the streaming budget and the audio bank, in the same shape on both hosts.

```ts
EngineProbe.memory(): ProbeMemory | null
```

**Returns** `ProbeMemory | null`: Null from a host that gives no figures.

Binding symbol `og_engine_probe_memory` · publish builds: dropped

```ogt
fn memory {
  doc "What the engine holds: the wasm linear memory on the web or the process's resident memory natively, the engine's counted heap, the world's entities, the streaming budget and the audio bank, in the same shape on both hosts."
  returns Option.ProbeMemory "Null from a host that gives no figures."
}
```

<sub>`api.ogt` line 24631</sub>

<a id="member-EngineProbe-describe"></a>

#### `EngineProbe.describe`

The frame of the current tick as data: the visible entities with screen boxes, depth order and occlusion from the entity-id pass, their names, archetypes and labels; the UI elements with rects and text; the camera. Answers without a renderer too (the pass runs on the CPU).

```ts
EngineProbe.describe(): Description
```

**Returns** `Description`.

Binding symbol `og_engine_probe_describe` · publish builds: dropped

```ogt
fn describe {
  doc "The frame of the current tick as data: the visible entities with screen boxes, depth order and occlusion from the entity-id pass, their names, archetypes and labels; the UI elements with rects and text; the camera. Answers without a renderer too (the pass runs on the CPU)."
  returns Description
}
```

<sub>`api.ogt` line 24635</sub>

<a id="member-EngineProbe-expect"></a>

#### `EngineProbe.expect`

Evaluate one expectation now (the forms of the command channel's `expect:`); the result is also kept for expectations().

```ts
EngineProbe.expect(what: string): ExpectResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `what` | `string` | no | The expectation, such as `entity:crate:within:0,0.4,-1,1,0.6,1`. |

**Returns** `ExpectResult`: A malformed expectation is a malformed call.

Binding symbol `og_engine_probe_expect` · publish builds: dropped

```ogt
fn expect {
  doc "Evaluate one expectation now (the forms of the command channel's `expect:`); the result is also kept for expectations()."
  param what String "The expectation, such as `entity:crate:within:0,0.4,-1,1,0.6,1`."
  returns ExpectResult "A malformed expectation is a malformed call."
}
```

<sub>`api.ogt` line 24639</sub>

<a id="member-EngineProbe-expectations"></a>

#### `EngineProbe.expectations`

Every expectation evaluated since the last call (scripts' `expect:` steps, programs, expect()), and the counts.

```ts
EngineProbe.expectations(): ExpectReport
```

**Returns** `ExpectReport`.

Binding symbol `og_engine_probe_expectations` · publish builds: dropped

```ogt
fn expectations {
  doc "Every expectation evaluated since the last call (scripts' `expect:` steps, programs, expect()), and the counts."
  returns ExpectReport
}
```

<sub>`api.ogt` line 24644</sub>

<a id="member-EngineProbe-assertions"></a>

#### `EngineProbe.assertions`

The run's assertions (og.assert, the default invariants, `invariant:` steps): every site with whether it fired, each violated one's first violation with its tick, entity, record, field and value, each's buckets, and what never fired. With take, the report is answered and a new one starts: the sites stay listed, unfired, so an independent run (another playtest, a sibling fork) inherits none of this one's violations or coverage.

```ts
EngineProbe.assertions(take?: boolean): AssertReport
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `take` | `boolean` | yes | Start a new report after answering this one. Default `false`. |

**Returns** `AssertReport`.

Binding symbol `og_engine_probe_assertions` · publish builds: dropped

```ogt
fn assertions {
  doc "The run's assertions (og.assert, the default invariants, `invariant:` steps): every site with whether it fired, each violated one's first violation with its tick, entity, record, field and value, each's buckets, and what never fired. With take, the report is answered and a new one starts: the sites stay listed, unfired, so an independent run (another playtest, a sibling fork) inherits none of this one's violations or coverage."
  param take bool "Start a new report after answering this one." {
    default false
  }
  returns AssertReport
}
```

<sub>`api.ogt` line 24648</sub>

<a id="member-EngineProbe-saveScene"></a>

#### `EngineProbe.saveScene`

Write every scene the world was assembled from back from the world (the boot scene's and each og.world.instantiate made): its entities' and resources' changed field lines and the fields set and patch tuned, keeping every other line, comment and the order; with spawned, the entities the game spawned under the boot scene's marked section. The probe writes the files only with write; otherwise the texts are returned.

```ts
EngineProbe.saveScene(write?: boolean, spawned?: boolean): SavedScene | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `write` | `boolean` | yes | Replace each scene's source file. Default `false`. |
| `spawned` | `boolean` | yes | Write the entities the game spawned at run time under the boot scene's marked section. Default `false`. |

**Returns** `SavedScene | null`: Null without a scene to write back (the reason is in the log).

Binding symbol `og_engine_probe_save_scene` · publish builds: dropped

```ogt
fn saveScene {
  doc "Write every scene the world was assembled from back from the world (the boot scene's and each og.world.instantiate made): its entities' and resources' changed field lines and the fields set and patch tuned, keeping every other line, comment and the order; with spawned, the entities the game spawned under the boot scene's marked section. The probe writes the files only with write; otherwise the texts are returned."
  param write bool "Replace each scene's source file." {
    default false
  }
  param spawned bool "Write the entities the game spawned at run time under the boot scene's marked section." {
    default false
  }
  returns Option.SavedScene "Null without a scene to write back (the reason is in the log)."
}
```

<sub>`api.ogt` line 24655</sub>

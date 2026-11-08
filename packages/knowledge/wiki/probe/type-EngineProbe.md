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

<sub>Module `probe` line 648</sub>

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

<sub>Module `probe` line 653</sub>

<a id="member-EngineProbe-capture"></a>

#### `EngineProbe.capture`

The frame of the current tick, rendered offscreen and read back as a PNG; null without a renderer (hello's tier `none`).

Size rule: width and height are both 0 (or absent), which is the render target's size, or both 1 to 4096, which renders that size with the same camera (the aspect follows the requested size); one zero and one not is a malformed call.

On the web the readback is asynchronous: through the request ring the answer arrives when the copy completes; a synchronous in-page call answers null on the first tick and the frame from then on (a capture is requested on tick N and answered on N + 1).

Also known as: screenshot, take a screenshot, grab the screen, frame capture

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
  aliases "screenshot, take a screenshot, grab the screen, frame capture"
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

<sub>Module `probe` line 658</sub>

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

<sub>Module `probe` line 672</sub>

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

<sub>Module `probe` line 679</sub>

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

<sub>Module `probe` line 683</sub>

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

<sub>Module `probe` line 687</sub>

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

<sub>Module `probe` line 692</sub>

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

<sub>Module `probe` line 699</sub>

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

<sub>Module `probe` line 704</sub>

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

<sub>Module `probe` line 711</sub>

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

<sub>Module `probe` line 715</sub>

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

<sub>Module `probe` line 719</sub>

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

<sub>Module `probe` line 724</sub>

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

<sub>Module `probe` line 728</sub>

<a id="member-EngineProbe-assertionEvents"></a>

#### `EngineProbe.assertionEvents`

The assertions' stream since the last call: each site's first firing, each each-bucket's first holding and each site's first violation, with its tick, in the order they happened (at most 4,096 between calls; more are counted in dropped). A tool that steps a run reads it after each advance and stops at the first violation, instead of reading the whole report.

```ts
EngineProbe.assertionEvents(): AssertEvents
```

**Returns** `AssertEvents`.

Binding symbol `og_engine_probe_assertion_events` · publish builds: dropped

```ogt
fn assertionEvents {
  doc "The assertions' stream since the last call: each site's first firing, each each-bucket's first holding and each site's first violation, with its tick, in the order they happened (at most 4,096 between calls; more are counted in dropped). A tool that steps a run reads it after each advance and stops at the first violation, instead of reading the whole report."
  returns AssertEvents
}
```

<sub>Module `probe` line 735</sub>

<a id="member-EngineProbe-observe"></a>

#### `EngineProbe.observe`

The game's reflected state as values: each of its resources' fields, and each of its components' fields on up to 256 entities by probe id (`{ tick, resources: { Score: { value } }, components: { Health: { hero: { hp } } } }`). Records names which (engine records too, `og.Transform`); empty is every record the game declared. What a search of the game's states abstracts its cells from (og explore).

```ts
EngineProbe.observe(records?: readonly string[] | null, types?: boolean): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `records` | `readonly string[] \| null` | yes | The records to read; null or empty: the game's own. |
| `types` | `boolean` | yes | Also answer `types`: each of those records' kind (component, resource or event) and fields with their type, min, max and variants, the events among them. Default `false`. |

**Returns** `unknown`.

Binding symbol `og_engine_probe_observe` · publish builds: dropped

```ogt
fn observe {
  doc "The game's reflected state as values: each of its resources' fields, and each of its components' fields on up to 256 entities by probe id (`{ tick, resources: { Score: { value } }, components: { Health: { hero: { hp } } } }`). Records names which (engine records too, `og.Transform`); empty is every record the game declared. What a search of the game's states abstracts its cells from (og explore)."
  param records Option.List.String "The records to read; null or empty: the game's own." {
    optional true
  }
  param types bool "Also answer `types`: each of those records' kind (component, resource or event) and fields with their type, min, max and variants, the events among them." {
    default false
  }
  returns Json
}
```

<sub>Module `probe` line 739</sub>

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

<sub>Module `probe` line 749</sub>

<a id="member-EngineProbe-pickUi"></a>

#### `EngineProbe.pickUi`

The UI element under a point of the play surface: what pick names in the UI, with the document line that built it and the style rules that dress it. A tool asks it first, then pick for the scene.

```ts
EngineProbe.pickUi(x: number, y: number): UiPickElement | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | Horizontal position in CSS pixels. |
| `y` | `number` | no | Vertical position in CSS pixels. |

**Returns** `UiPickElement | null`: Null when no UI element that shows something or takes input is under the point.

Binding symbol `og_engine_probe_pick_ui` · publish builds: dropped

```ogt
fn pickUi {
  doc "The UI element under a point of the play surface: what pick names in the UI, with the document line that built it and the style rules that dress it. A tool asks it first, then pick for the scene."
  param x f32 "Horizontal position in CSS pixels."
  param y f32 "Vertical position in CSS pixels."
  returns Option.UiPickElement "Null when no UI element that shows something or takes input is under the point."
}
```

<sub>Module `probe` line 759</sub>

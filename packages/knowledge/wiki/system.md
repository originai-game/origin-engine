<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `system` module

Systems: functions the engine calls once per tick with their matched archetypes.

A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name.

Its query is every component of its reads and writes together: an entity matches only when it has all of them (and none of `without`). A component declared to read one entity with world.get (the player's car) therefore narrows the loop to entities that have it too, often to none, silently. world.query is refused inside a system, by design (a system reaches only what it declared, so its access is known before it runs): loop over each archetype in a system of its own, declaring only that archetype's components, and carry what crosses between systems in a resource.

Resources and events it reaches through the world must be declared too: in a dev or release build (og run, og test) an undeclared read or write throws with the system's name and the record and fails the tick; a publish build refuses it and logs it once per system in the engine log, and the game keeps running. og.Rng among the reads may fork (a fork reads the seed only and leaves the stream as it is); a draw moves the stream, so it needs og.Rng among the writes.

- **TypeScript**: functions are named exports of the package: `import { … } from "@origingame/engine"`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-core`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module system {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1236</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`StateWhen`](#type-StateWhen) | enum | When a system bound to a state machine's value runs. |
| [`StateDesc`](#type-StateDesc) | struct | A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. Its value is world state: saves, the hash and replays carry it. |
| [`RunCondition`](#type-RunCondition) | callback | Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. It may not change the world. |
| [`SystemDesc`](#type-SystemDesc) | desc struct | A system's declared access and order. |
| [`HookEvent`](#type-HookEvent) | enum | What a hook reacts to. |
| [`HookDesc`](#type-HookDesc) | desc struct | A hook's event and declared access. |
| [`HookRun`](#type-HookRun) | callback | A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), in the order hooks are listed in GameDesc.hooks. Its own structural changes apply when it returns and may run hooks in turn (16 deep at most). A restore runs none. |
| [`Hook`](#type-Hook) | interface | A declared hook. |
| [`defineHook`](#fn-defineHook) | function | Declare a hook on a component; list it in GameDesc.hooks. |
| [`SystemRun`](#type-SystemRun) | callback | A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows. |
| [`System`](#type-System) | interface | A registered system. |
| [`defineSystem`](#fn-defineSystem) | function | Declare a system. |

## Functions

<a id="fn-defineHook"></a>

### `defineHook`

Declare a hook on a component; list it in GameDesc.hooks.

```ts
defineHook<C extends AnyComponent>(name: string, component: C, desc: HookDesc, run: HookRun): Hook
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The hook's unique name. |
| `component` | `C` | no | The component it reacts to. |
| `desc` | `HookDesc` | no | Add or remove, and its access. |
| `run` | `HookRun` | no | The body. |

**Returns** `Hook`.

Binding symbol `og_system_define_hook` · publish builds: kept

```ogt
fn defineHook {
  doc "Declare a hook on a component; list it in GameDesc.hooks."
  generic C AnyComponent
  param name String "The hook's unique name."
  param component C "The component it reacts to."
  param desc HookDesc "Add or remove, and its access."
  param run HookRun "The body."
  returns Hook
}
```

<sub>`api.ogt` line 1308</sub>

<a id="fn-defineSystem"></a>

### `defineSystem`

Declare a system.

```ts
defineSystem(name: string, desc: SystemDesc, run: SystemRun): System
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The system's unique name, PascalCase. |
| `desc` | `SystemDesc` | no | Reads, writes and order. |
| `run` | `SystemRun` | no | The body. |

**Returns** `System`.

Binding symbol `og_system_define_system` · publish builds: kept

```ogt
fn defineSystem {
  doc "Declare a system."
  param name String "The system's unique name, PascalCase."
  param desc SystemDesc "Reads, writes and order."
  param run SystemRun "The body."
  returns System
}
```

<sub>`api.ogt` line 1332</sub>

## Interfaces

<a id="type-Hook"></a>

### `Hook` (interface)

A declared hook.

```ogt
interface Hook {
  doc "A declared hook."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1303</sub>

<a id="member-Hook-name"></a>

#### `Hook.name` (property)

The hook's unique name.

```ts
readonly name: string
```

Binding symbol `og_hook_get_name` · publish builds: kept

```ogt
prop name String "The hook's unique name."
```

<sub>`api.ogt` line 1305</sub>

<a id="type-System"></a>

### `System` (interface)

A registered system.

```ogt
interface System {
  doc "A registered system."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1325</sub>

<a id="member-System-name"></a>

#### `System.name` (property)

The system's unique name.

```ts
readonly name: string
```

Binding symbol `og_system_get_name` · publish builds: kept

```ogt
prop name String "The system's unique name."
```

<sub>`api.ogt` line 1327</sub>

<a id="member-System-reads"></a>

#### `System.reads` (property)

What it reads.

```ts
readonly reads: readonly AnyReflected[]
```

Binding symbol `og_system_get_reads` · publish builds: kept

```ogt
prop reads List.AnyReflected "What it reads."
```

<sub>`api.ogt` line 1328</sub>

<a id="member-System-writes"></a>

#### `System.writes` (property)

What it writes.

```ts
readonly writes: readonly AnyReflected[]
```

Binding symbol `og_system_get_writes` · publish builds: kept

```ogt
prop writes List.AnyReflected "What it writes."
```

<sub>`api.ogt` line 1329</sub>

## Types

<a id="type-StateWhen"></a>

### `StateWhen` (enum)

When a system bound to a state machine's value runs.

A string: one of the variants below. The zero value is `"during"`.

| Variant | Doc |
|---|---|
| `"during"` | Every tick while the state machine holds the value. |
| `"enter"` | Once, as the tick begins, when the machine enters the value (the first tick, for its initial value). |
| `"exit"` | Once, as the tick begins, when the machine leaves the value. |

```ogt
enum StateWhen {
  doc "When a system bound to a state machine's value runs."
  variant during "Every tick while the state machine holds the value."
  variant enter "Once, as the tick begins, when the machine enters the value (the first tick, for its initial value)."
  variant exit "Once, as the tick begins, when the machine leaves the value."
}
```

<sub>`api.ogt` line 1244</sub>

<a id="type-StateDesc"></a>

### `StateDesc` (struct)

A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. Its value is world state: saves, the hash and replays carry it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its unique name. |
| `values` | `readonly string[]` | `[]` | — | — | — | Its values, at least one, each unique. |
| `initial` | `string` | `""` | — | — | optional | The value it starts in; empty or omitted: the first. |

```ogt
struct StateDesc {
  doc "A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. Its value is world state: saves, the hash and replays carry it."
  field name String "Its unique name."
  field values List.String "Its values, at least one, each unique."
  field initial String "The value it starts in; empty or omitted: the first." {
    optional true
  }
}
```

<sub>`api.ogt` line 1251</sub>

<a id="type-RunCondition"></a>

### `RunCondition` (callback)

Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. It may not change the world.

Game code supplies this function.

```ts
type RunCondition = (world: World) => boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, read only. |

```ogt
callback RunCondition {
  doc "Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. It may not change the world."
  param world World "The world, read only."
  returns bool
}
```

<sub>`api.ogt` line 1260</sub>

<a id="type-SystemDesc"></a>

### `SystemDesc` (desc struct)

A system's declared access and order.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reads` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components. |
| `writes` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components a matched entity must not have. |
| `after` | `readonly string[]` | `[]` | — | — | — | Names of systems that must run before this one. |
| `before` | `readonly string[]` | `[]` | — | — | — | Names of systems that must run after this one. |
| `changed` | `readonly AnyComponent[]` | `[]` | — | — | — | Change filters on the query: components (required, as reads) whose value changed since this system last ran. A change is a value that differs from the one last compared, compared after each system and hook for what it writes and as each tick begins for what was written between ticks: counted in system runs, never by a clock, so saves and replays carry it. The archetypes the query hands carry the rows that pass every filter (ArchetypeView.rows). |
| `added` | `readonly AnyComponent[]` | `[]` | — | — | — | Components (required, as reads) added since this system last ran: spawned with, or inserted into an entity without them. |
| `removed` | `readonly AnyComponent[]` | `[]` | — | — | — | Components the entity lost since this system last ran (a despawned entity: world.removed). |
| `state` | `string` | `""` | — | — | — | A state machine (GameDesc.states) the system is bound to; empty: none. |
| `value` | `string` | `""` | — | — | — | The value of `state` it is bound to. |
| `when` | `StateWhen` | `"during"` | — | — | — | When it runs: during the value, or once on entering or leaving it. |
| `runIf` | `RunCondition \| null` | `null` | — | — | — | A run condition: the system runs only on ticks it answers true. A skipped tick is not a run: the change filters keep reading from the last one. |

```ogt
struct SystemDesc {
  doc "A system's declared access and order."
  desc true
  field reads List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components."
  field writes List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'."
  field without List.AnyComponent "Components a matched entity must not have."
  field after List.String "Names of systems that must run before this one."
  field before List.String "Names of systems that must run after this one."
  field changed List.AnyComponent "Change filters on the query: components (required, as reads) whose value changed since this system last ran. A change is a value that differs from the one last compared, compared after each system and hook for what it writes and as each tick begins for what was written between ticks: counted in system runs, never by a clock, so saves and replays carry it. The archetypes the query hands carry the rows that pass every filter (ArchetypeView.rows)."
  field added List.AnyComponent "Components (required, as reads) added since this system last ran: spawned with, or inserted into an entity without them."
  field removed List.AnyComponent "Components the entity lost since this system last ran (a despawned entity: world.removed)."
  field state String "A state machine (GameDesc.states) the system is bound to; empty: none."
  field value String "The value of `state` it is bound to."
  field when StateWhen "When it runs: during the value, or once on entering or leaving it."
  field runIf Option.RunCondition "A run condition: the system runs only on ticks it answers true. A skipped tick is not a run: the change filters keep reading from the last one."
}
```

<sub>`api.ogt` line 1266</sub>

<a id="type-HookEvent"></a>

### `HookEvent` (enum)

What a hook reacts to.

A string: one of the variants below. The zero value is `"add"`.

| Variant | Doc |
|---|---|
| `"add"` | The component was added (spawned with, or inserted into an entity without it): the hook runs after it is in place. |
| `"remove"` | The component is being removed (removed, or its entity despawned): the hook runs before it goes, and reads it. |

```ogt
enum HookEvent {
  doc "What a hook reacts to."
  variant add "The component was added (spawned with, or inserted into an entity without it): the hook runs after it is in place."
  variant remove "The component is being removed (removed, or its entity despawned): the hook runs before it goes, and reads it."
}
```

<sub>`api.ogt` line 1283</sub>

<a id="type-HookDesc"></a>

### `HookDesc` (desc struct)

A hook's event and declared access.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `on` | `HookEvent` | `"add"` | — | — | — | Add or remove. |
| `reads` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | What it reads, as a system declares it. |
| `writes` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | What it writes, as a system declares it. |

```ogt
struct HookDesc {
  doc "A hook's event and declared access."
  desc true
  field on HookEvent "Add or remove."
  field reads List.OneOf.AnyReflected.AnyEvent "What it reads, as a system declares it."
  field writes List.OneOf.AnyReflected.AnyEvent "What it writes, as a system declares it."
}
```

<sub>`api.ogt` line 1289</sub>

<a id="type-HookRun"></a>

### `HookRun` (callback)

A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), in the order hooks are listed in GameDesc.hooks. Its own structural changes apply when it returns and may run hooks in turn (16 deep at most). A restore runs none.

Game code supplies this function.

```ts
type HookRun = (world: World, entity: Entity) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, as a system's. |
| `entity` | `Entity` | no | The entity. |

```ogt
callback HookRun {
  doc "A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), in the order hooks are listed in GameDesc.hooks. Its own structural changes apply when it returns and may run hooks in turn (16 deep at most). A restore runs none."
  param world World "The world, as a system's."
  param entity Entity "The entity."
}
```

<sub>`api.ogt` line 1297</sub>

<a id="type-SystemRun"></a>

### `SystemRun` (callback)

A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows.

Game code supplies this function.

```ts
type SystemRun = (q: Query, time: Time, world: World) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `q` | `Query` | no | The archetypes matching the declared components. |
| `time` | `Time` | no | The Time resource for this tick. |
| `world` | `World` | no | The world, for resources, events and structural changes. |

```ogt
callback SystemRun {
  doc "A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows."
  param q Query "The archetypes matching the declared components."
  param time Time "The Time resource for this tick."
  param world World "The world, for resources, events and structural changes."
}
```

<sub>`api.ogt` line 1318</sub>

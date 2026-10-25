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

<sub>`api.ogt` line 1008</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`SystemDesc`](#type-SystemDesc) | desc struct | A system's declared access and order. |
| [`SystemRun`](#type-SystemRun) | callback | A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows. |
| [`System`](#type-System) | interface | A registered system. |
| [`defineSystem`](#fn-defineSystem) | function | Declare a system. |

## Functions

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

<sub>`api.ogt` line 1040</sub>

## Interfaces

<a id="type-System"></a>

### `System` (interface)

A registered system.

```ogt
interface System {
  doc "A registered system."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1033</sub>

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

<sub>`api.ogt` line 1035</sub>

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

<sub>`api.ogt` line 1036</sub>

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

<sub>`api.ogt` line 1037</sub>

## Types

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

```ogt
struct SystemDesc {
  doc "A system's declared access and order."
  desc true
  field reads List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components."
  field writes List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'."
  field without List.AnyComponent "Components a matched entity must not have."
  field after List.String "Names of systems that must run before this one."
  field before List.String "Names of systems that must run after this one."
}
```

<sub>`api.ogt` line 1016</sub>

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

<sub>`api.ogt` line 1026</sub>

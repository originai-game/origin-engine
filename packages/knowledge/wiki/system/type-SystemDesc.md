<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-SystemDesc"></a>

### `SystemDesc` (desc struct)

A system's declared access and order.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reads` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components, so one read for a single entity with world.get (the player's car) narrows the loop, often to none. |
| `writes` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components a matched entity must not have. |
| `after` | `readonly string[]` | `[]` | — | — | — | Names of systems that must run before this one. |
| `before` | `readonly string[]` | `[]` | — | — | — | Names of systems that must run after this one. |
| `changed` | `readonly AnyComponent[]` | `[]` | — | — | — | Change filters: components (required, as reads) whose value changed since the system last ran, compared after each system and hook for its writes, as each tick begins for those between ticks: counted in system runs, never by a clock, so saves and replays carry it. Passing rows: ArchetypeView.rows. |
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
  field reads List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components, so one read for a single entity with world.get (the player's car) narrows the loop, often to none."
  field writes List.OneOf.AnyReflected.AnyEvent "Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'."
  field without List.AnyComponent "Components a matched entity must not have."
  field after List.String "Names of systems that must run before this one."
  field before List.String "Names of systems that must run after this one."
  field changed List.AnyComponent "Change filters: components (required, as reads) whose value changed since the system last ran, compared after each system and hook for its writes, as each tick begins for those between ticks: counted in system runs, never by a clock, so saves and replays carry it. Passing rows: ArchetypeView.rows."
  field added List.AnyComponent "Components (required, as reads) added since this system last ran: spawned with, or inserted into an entity without them."
  field removed List.AnyComponent "Components the entity lost since this system last ran (a despawned entity: world.removed)."
  field state String "A state machine (GameDesc.states) the system is bound to; empty: none."
  field value String "The value of `state` it is bound to."
  field when StateWhen "When it runs: during the value, or once on entering or leaving it."
  field runIf Option.RunCondition "A run condition: the system runs only on ticks it answers true. A skipped tick is not a run: the change filters keep reading from the last one."
}
```

<sub>Module `system` line 29</sub>

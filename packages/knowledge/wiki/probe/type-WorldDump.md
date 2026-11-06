<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

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

<sub>`api.ogt` line 24318</sub>

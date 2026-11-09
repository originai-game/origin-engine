<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

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

<sub>Module `probe` line 387</sub>

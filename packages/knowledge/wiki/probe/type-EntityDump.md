<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-EntityDump"></a>

### `EntityDump` (struct)

One entity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `name` | `string \| null` | `null` | — | — | — | Its name. |
| `components` | `Readonly<Record<string, unknown>>` | `{}` | — | — | — | Each component's field values, by component name. |
| `prefab` | `EntityPrefab` | `{ prefab: "", node: "", root: "", overrides: [] }` | — | — | optional | Where it came from, for an entity placed from a prefab; absent otherwise. |

```ogt
struct EntityDump {
  doc "One entity."
  field id String "Its probe id."
  field name Option.String "Its name."
  field components Map.Json "Each component's field values, by component name."
  field prefab EntityPrefab "Where it came from, for an entity placed from a prefab; absent otherwise." {
    optional true
  }
}
```

<sub>`api.ogt` line 24249</sub>

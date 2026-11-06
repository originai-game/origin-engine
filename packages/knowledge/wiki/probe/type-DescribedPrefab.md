<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedPrefab"></a>

### `DescribedPrefab` (struct)

The instances of one prefab in the world.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prefab` | `string` | `""` | — | — | — | The prefab's asset id. |
| `instances` | `readonly string[]` | `[]` | — | — | — | Each instance root's probe id, in entity order. |

```ogt
struct DescribedPrefab {
  doc "The instances of one prefab in the world."
  field prefab String "The prefab's asset id."
  field instances List.String "Each instance root's probe id, in entity order."
}
```

<sub>Module `probe` line 345</sub>

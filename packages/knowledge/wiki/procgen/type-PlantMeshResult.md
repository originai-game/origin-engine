<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantMeshResult"></a>

### `PlantMeshResult` (struct)

Generated procedural plant meshes and statistics.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wood` | `PlantMeshPart` | — | — | — | — | Trunk and branch geometry. |
| `leaves` | `PlantMeshPart \| null` | `null` | — | — | — | Leaves and flowers geometry, or null if none grew. |
| `branches` | `number` | `0` | — | — | — | Total branch count generated. |
| `leafCount` | `number` | `0` | — | — | — | Total leaf count generated. |
| `flowerCount` | `number` | `0` | — | — | — | Total flower count generated. |

```ogt
struct PlantMeshResult {
  doc "Generated procedural plant meshes and statistics."
  field wood PlantMeshPart "Trunk and branch geometry."
  field leaves Option.PlantMeshPart "Leaves and flowers geometry, or null if none grew."
  field branches u32 "Total branch count generated."
  field leafCount u32 "Total leaf count generated."
  field flowerCount u32 "Total flower count generated."
}
```

<sub>Module `procgen` line 830</sub>

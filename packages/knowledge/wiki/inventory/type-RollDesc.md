<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="type-RollDesc"></a>

### `RollDesc` (struct)

What og.inventory.roll draws.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entries` | `readonly LootEntry[]` | `[]` | — | — | — | The loot table's lines. |
| `rolls` | `number` | `0` | — | — | — | How many draws; 0 draws nothing. |

```ogt
struct RollDesc {
  doc "What og.inventory.roll draws."
  field entries List.LootEntry "The loot table's lines."
  field rolls u32 "How many draws; 0 draws nothing."
}
```

<sub>Module `inventory` line 69</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="type-LootEntry"></a>

### `LootEntry` (struct)

One line of a loot table.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `item` | `TableRow` | `0` | — | — | — | The row it gives. |
| `weight` | `number` | `0` | — | — | — | Its share of the draws; 0 is never drawn. |
| `min` | `number` | `0` | — | — | — | The fewest it gives when drawn. |
| `max` | `number` | `0` | — | — | — | The most it gives when drawn; under min gives min. |

```ogt
struct LootEntry {
  doc "One line of a loot table."
  field item TableRow "The row it gives."
  field weight f64 "Its share of the draws; 0 is never drawn."
  field min u32 "The fewest it gives when drawn."
  field max u32 "The most it gives when drawn; under min gives min."
}
```

<sub>Module `inventory` line 61</sub>

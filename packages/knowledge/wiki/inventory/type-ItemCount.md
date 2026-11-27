<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="type-ItemCount"></a>

### `ItemCount` (struct)

Some of one row.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `item` | `TableRow` | `0` | — | — | — | The row. |
| `count` | `number` | `0` | — | — | — | How many. |

```ogt
struct ItemCount {
  doc "Some of one row."
  field item TableRow "The row."
  field count u32 "How many."
}
```

<sub>Module `inventory` line 55</sub>

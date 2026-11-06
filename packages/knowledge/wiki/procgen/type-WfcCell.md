<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-WfcCell"></a>

### `WfcCell` (struct)

A cell fixed before generation starts (an entrance, a river's source).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column. |
| `y` | `number` | `0` | — | — | — | The row. |
| `tile` | `number` | `0` | — | — | — | The tile's index in WfcDesc.tiles. |
| `rotation` | `number` | `0` | — | — | — | Quarter turns clockwise, 0 to 3. |

```ogt
struct WfcCell {
  doc "A cell fixed before generation starts (an entrance, a river's source)."
  field x u32 "The column."
  field y u32 "The row."
  field tile u32 "The tile's index in WfcDesc.tiles."
  field rotation u32 "Quarter turns clockwise, 0 to 3."
}
```

<sub>`api.ogt` line 1967</sub>

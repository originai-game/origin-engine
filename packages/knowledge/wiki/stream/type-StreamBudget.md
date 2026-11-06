<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="type-StreamBudget"></a>

### `StreamBudget` (struct)

The streaming memory budget and every group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `budget` | `number` | `0` | — | `bytes` | — | The bytes resident and loading groups may take. |
| `used` | `number` | `0` | — | `bytes` | — | The bytes they take now, with what unloaded groups keep for the frame. |
| `kept` | `number` | `0` | — | `bytes` | — | Of `used`, the bytes of assets kept for the frame after their group unloaded (an array a sprite still draws, the map the frame is lit by, ...). |
| `peak` | `number` | `0` | — | `bytes` | — | The most they ever took (never above the budget at the time). |
| `groups` | `readonly StreamStatus[]` | `[]` | — | — | — | Every group, by name. |

```ogt
struct StreamBudget {
  doc "The streaming memory budget and every group."
  field budget f64 "The bytes resident and loading groups may take." {
    unit bytes
  }
  field used f64 "The bytes they take now, with what unloaded groups keep for the frame." {
    unit bytes
  }
  field kept f64 "Of `used`, the bytes of assets kept for the frame after their group unloaded (an array a sprite still draws, the map the frame is lit by, ...)." {
    unit bytes
  }
  field peak f64 "The most they ever took (never above the budget at the time)." {
    unit bytes
  }
  field groups List.StreamStatus "Every group, by name."
}
```

<sub>`api.ogt` line 25471</sub>

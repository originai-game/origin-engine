<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeStreamMemory"></a>

### `ProbeStreamMemory` (struct)

The streaming memory budget (og.stream.budget): what the admitted groups' assets take.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `budget` | `number` | `0` | — | `bytes` | — | The budget. |
| `used` | `number` | `0` | — | `bytes` | — | What the admitted groups take now. |
| `peak` | `number` | `0` | — | `bytes` | — | The most they took at once. |

```ogt
struct ProbeStreamMemory {
  doc "The streaming memory budget (og.stream.budget): what the admitted groups' assets take."
  field budget f64 "The budget." {
    unit bytes
  }
  field used f64 "What the admitted groups take now." {
    unit bytes
  }
  field peak f64 "The most they took at once." {
    unit bytes
  }
}
```

<sub>Module `probe` line 209</sub>

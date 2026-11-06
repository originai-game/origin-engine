<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SweepFrame"></a>

### `SweepFrame` (struct)

One frame of a sweep.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `unknown` | `null` | — | — | — | The value patched in. |
| `tick` | `number` | `0` | — | — | — | The tick captured. |
| `png` | `string` | `""` | — | — | — | The PNG, base64. |

```ogt
struct SweepFrame {
  doc "One frame of a sweep."
  field value Json "The value patched in."
  field tick u32 "The tick captured."
  field png String "The PNG, base64."
}
```

<sub>`api.ogt` line 24290</sub>

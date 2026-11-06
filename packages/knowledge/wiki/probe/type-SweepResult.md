<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SweepResult"></a>

### `SweepResult` (struct)

A sweep's frames, one per value.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `frames` | `readonly SweepFrame[]` | `[]` | — | — | — | The frames. |

```ogt
struct SweepResult {
  doc "A sweep's frames, one per value."
  field frames List.SweepFrame "The frames."
}
```

<sub>Module `probe` line 259</sub>

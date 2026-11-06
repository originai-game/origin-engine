<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeTiming"></a>

### `ProbeTiming` (struct)

One named span of the last tick or frame.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A system's name, or a render pass's (`Main`, `Cascade0`, `Bloom`). |
| `ms` | `number` | `0` | — | `ms` | — | Its time. |

```ogt
struct ProbeTiming {
  doc "One named span of the last tick or frame."
  field name String "A system's name, or a render pass's (`Main`, `Cascade0`, `Bloom`)."
  field ms f32 "Its time." {
    unit ms
  }
}
```

<sub>`api.ogt` line 24125</sub>

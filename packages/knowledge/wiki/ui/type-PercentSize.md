<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PercentSize"></a>

### `PercentSize` (struct)

A size relative to the parent.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fraction` | `number` | `0` | — | `ratio` | — | The fraction of the parent's inner size. |

```ogt
struct PercentSize {
  doc "A size relative to the parent."
  field fraction f32 "The fraction of the parent's inner size." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 15802</sub>

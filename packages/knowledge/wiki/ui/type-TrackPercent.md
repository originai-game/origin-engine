<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TrackPercent"></a>

### `TrackPercent` (struct)

A fraction of the grid's inner size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fraction` | `number` | `0` | — | `ratio` | — | The fraction. |

```ogt
struct TrackPercent {
  doc "A fraction of the grid's inner size."
  field fraction f32 "The fraction." {
    unit ratio
  }
}
```

<sub>Module `ui` line 1958</sub>

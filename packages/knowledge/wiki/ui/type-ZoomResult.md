<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ZoomResult"></a>

### `ZoomResult` (struct)

Where a zoom view is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `zoom` | `number` | `0` | — | — | — | Its zoom: 1 shows the whole picture, 2 half of it each way. |
| `centerX` | `number` | `0` | — | — | — | The middle of what it shows, as a fraction of the picture's width (0.5 at rest). |
| `centerY` | `number` | `0` | — | — | — | The middle of what it shows, as a fraction of the picture's height (0.5 at rest). |

```ogt
struct ZoomResult {
  doc "Where a zoom view is."
  field zoom f32 "Its zoom: 1 shows the whole picture, 2 half of it each way."
  field centerX f32 "The middle of what it shows, as a fraction of the picture's width (0.5 at rest)."
  field centerY f32 "The middle of what it shows, as a fraction of the picture's height (0.5 at rest)."
}
```

<sub>`api.ogt` line 15987</sub>

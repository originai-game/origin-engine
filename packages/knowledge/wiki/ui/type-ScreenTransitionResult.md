<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScreenTransitionResult"></a>

### `ScreenTransitionResult` (struct)

Where a screen transition is this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shown` | `string` | `""` | — | — | — | The screen to build this tick: the old one until the cover closes, then the new one. |
| `covering` | `boolean` | `false` | — | — | — | The cover is closing over the old screen. |
| `revealing` | `boolean` | `false` | — | — | — | The cover is opening on the new screen. |
| `cover` | `number` | `0` | — | `ratio` | — | How much of the surface is covered, 0 to 1. |

```ogt
struct ScreenTransitionResult {
  doc "Where a screen transition is this tick."
  field shown String "The screen to build this tick: the old one until the cover closes, then the new one."
  field covering bool "The cover is closing over the old screen."
  field revealing bool "The cover is opening on the new screen."
  field cover f32 "How much of the surface is covered, 0 to 1." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 17654</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TaaHistory"></a>

### `TaaHistory` (enum)

How TAA resamples its history each frame, as og-render's TaaHistory.

A string: one of the variants below. The zero value is `"bilinear"`.

| Variant | Doc |
|---|---|
| `"bilinear"` | One bilinear tap: cheap, slightly soft while the camera moves. |
| `"catmullRom"` | A Catmull-Rom filter over 4x4 texels (9 taps): the history stays sharp under camera motion. |

```ogt
enum TaaHistory {
  doc "How TAA resamples its history each frame, as og-render's TaaHistory."
  variant bilinear "One bilinear tap: cheap, slightly soft while the camera moves."
  variant catmullRom "A Catmull-Rom filter over 4x4 texels (9 taps): the history stays sharp under camera motion."
}
```

<sub>Module `render` line 79</sub>

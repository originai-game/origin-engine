<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="type-CameraGateFit"></a>

### `CameraGateFit` (enum)

Fit a sensor to a target aspect without stretching pixels.

A string: one of the variants below. The zero value is `"vertical"`.

| Variant | Doc |
|---|---|
| `"vertical"` | Keep sensor height; width follows the target aspect. |
| `"horizontal"` | Keep sensor width; height follows the target aspect. |
| `"fill"` | Crop to fill: use the smaller fitted height. |
| `"overscan"` | Show the whole gate: use the larger fitted height. |

```ogt
enum CameraGateFit {
  doc "Fit a sensor to a target aspect without stretching pixels."
  variant vertical "Keep sensor height; width follows the target aspect."
  variant horizontal "Keep sensor width; height follows the target aspect."
  variant fill "Crop to fill: use the smaller fitted height."
  variant overscan "Show the whole gate: use the larger fitted height."
}
```

<sub>Module `camera` line 200</sub>

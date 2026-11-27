<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="type-CameraExposureMode"></a>

### `CameraExposureMode` (enum)

Manual exposure source; existing artistic multipliers remain compensation.

A string: one of the variants below. The zero value is `"artistic"`.

| Variant | Doc |
|---|---|
| `"artistic"` | Keep existing artistic exposure. |
| `"settings"` | Derive exposure from aperture, shutterSpeed and iso. |
| `"ev100"` | Use explicit ev100, including zero EV. |

```ogt
enum CameraExposureMode {
  doc "Manual exposure source; existing artistic multipliers remain compensation."
  variant artistic "Keep existing artistic exposure."
  variant settings "Derive exposure from aperture, shutterSpeed and iso."
  variant ev100 "Use explicit ev100, including zero EV."
}
```

<sub>Module `camera` line 220</sub>

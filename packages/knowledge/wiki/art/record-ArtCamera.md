<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="record-ArtCamera"></a>

### `og.ArtCamera` (resource)

The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. Every field is no constraint at its zero value (pitch and yaw matter only with fixedAngle).

Token `og.ArtCamera`, registered under the schema name `og.ArtCamera`; its value type is `ArtCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `projection` | `ArtProjection` | `"any"` | — | — | — | The projection: any (no constraint), perspective or orthographic. |
| `fixedAngle` | `boolean` | `false` | — | — | — | The camera keeps one angle, pitch and yaw: false is no constraint. |
| `pitch` | `number` | `0` | -90 … 90 | `deg` | — | With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric). |
| `yaw` | `number` | `0` | -180 … 180 | `deg` | — | With fixedAngle: its turn about the vertical axis (45 for isometric). |
| `fovMin` | `number` | `0` | 0 … 179 | `deg` | — | The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `fovMax` | `number` | `0` | 0 … 179 | `deg` | — | The widest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `pixelPerfect` | `boolean` | `false` | — | — | — | A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls. |

```ogt
resource ArtCamera {
  doc "The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. Every field is no constraint at its zero value (pitch and yaw matter only with fixedAngle)."
  field projection ArtProjection "The projection: any (no constraint), perspective or orthographic."
  field fixedAngle bool "The camera keeps one angle, pitch and yaw: false is no constraint."
  field pitch f32 "With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric)." {
    min -90
    max 90
    unit deg
  }
  field yaw f32 "With fixedAngle: its turn about the vertical axis (45 for isometric)." {
    min -180
    max 180
    unit deg
  }
  field fovMin f32 "The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit." {
    min 0
    max 179
    unit deg
  }
  field fovMax f32 "The widest vertical field of view a perspective camera of the style takes; 0 is no limit." {
    min 0
    max 179
    unit deg
  }
  field pixelPerfect bool "A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls."
}
```

<sub>`api.ogt` line 26743</sub>

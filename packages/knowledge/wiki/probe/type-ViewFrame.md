<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ViewFrame"></a>

### `ViewFrame` (struct)

One viewpoint's frame of captureViews.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `view` | `number` | `0` | — | — | — | Its index in the views asked. |
| `width` | `number` | `0` | — | — | — | Width in pixels. |
| `height` | `number` | `0` | — | — | — | Height in pixels. |
| `scale` | `number` | `0` | — | — | — | As Capture.scale. |
| `png` | `string` | `""` | — | — | — | The PNG, base64. |

```ogt
struct ViewFrame {
  doc "One viewpoint's frame of captureViews."
  field view u32 "Its index in the views asked."
  field width u32 "Width in pixels."
  field height u32 "Height in pixels."
  field scale f32 "As Capture.scale."
  field png String "The PNG, base64."
}
```

<sub>Module `probe` line 220</sub>

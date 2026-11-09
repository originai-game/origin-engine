<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Capture"></a>

### `Capture` (struct)

A captured frame.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the frame shows. |
| `width` | `number` | `0` | — | — | — | Width in pixels. |
| `height` | `number` | `0` | — | — | — | Height in pixels. |
| `scale` | `number` | `1` | — | — | — | The PNG's pixels per CSS pixel of the play surface: a point of pick or touch_move_to is (x / scale, y / scale) of the image's pixel, offset by region's corner. |
| `region` | `ScreenBox \| null` | `null` | — | — | — | The box of the play surface the PNG shows (CSS px); null for the whole frame. |
| `png` | `string` | `""` | — | — | — | The PNG, base64. |

```ogt
struct Capture {
  doc "A captured frame."
  field tick u32 "The tick the frame shows."
  field width u32 "Width in pixels."
  field height u32 "Height in pixels."
  field scale f32 "The PNG's pixels per CSS pixel of the play surface: a point of pick or touch_move_to is (x / scale, y / scale) of the image's pixel, offset by region's corner." {
    default 1
  }
  field region Option.ScreenBox "The box of the play surface the PNG shows (CSS px); null for the whole frame."
  field png String "The PNG, base64."
}
```

<sub>Module `probe` line 175</sub>

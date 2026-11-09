<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-MarkedCapture"></a>

### `MarkedCapture` (struct)

A capture with numbered boxes drawn over its UI elements and entities, and what each number is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the frame shows. |
| `width` | `number` | `0` | — | — | — | Width in pixels. |
| `height` | `number` | `0` | — | — | — | Height in pixels. |
| `scale` | `number` | `0` | — | — | — | The PNG's pixels per CSS pixel of the play surface. |
| `png` | `string` | `""` | — | — | — | The PNG with the marks drawn, base64. |
| `marks` | `readonly Mark[]` | `[]` | — | — | — | Each number on the image: the UI elements first in reading order (top to bottom, then left to right), then the entities nearest first. |
| `omitted` | `number` | `0` | — | — | — | Marks left out beyond max. |

```ogt
struct MarkedCapture {
  doc "A capture with numbered boxes drawn over its UI elements and entities, and what each number is."
  field tick u32 "The tick the frame shows."
  field width u32 "Width in pixels."
  field height u32 "Height in pixels."
  field scale f32 "The PNG's pixels per CSS pixel of the play surface."
  field png String "The PNG with the marks drawn, base64."
  field marks List.Mark "Each number on the image: the UI elements first in reading order (top to bottom, then left to right), then the entities nearest first."
  field omitted u32 "Marks left out beyond max."
}
```

<sub>Module `probe` line 240</sub>

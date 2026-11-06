<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-FlipbookInfo"></a>

### `FlipbookInfo` (struct)

What og.load.flipbook registered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sheet` | `FlipbookSheet` | `0` | — | — | — | What Flipbook.sheet takes. |
| `texture` | `Texture` | `0` | — | — | — | The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`); 0 for an `array true` sheet. |
| `array` | `TextureArray` | `0` | — | — | — | An `array true` sheet's texture array (its frames as layers, loaded with it); 0 otherwise. |
| `clips` | `readonly FlipbookClipInfo[]` | `[]` | — | — | — | Its clips, in sheet order. |
| `frames` | `number` | `0` | — | — | — | Frames in the sheet. |
| `frameWidth` | `number` | `0` | — | — | — | The first frame's width in pixels. |
| `frameHeight` | `number` | `0` | — | — | — | The first frame's height in pixels. |

```ogt
struct FlipbookInfo {
  doc "What og.load.flipbook registered."
  field sheet FlipbookSheet "What Flipbook.sheet takes."
  field texture Texture "The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`); 0 for an `array true` sheet."
  field array TextureArray "An `array true` sheet's texture array (its frames as layers, loaded with it); 0 otherwise."
  field clips List.FlipbookClipInfo "Its clips, in sheet order."
  field frames u32 "Frames in the sheet."
  field frameWidth u32 "The first frame's width in pixels."
  field frameHeight u32 "The first frame's height in pixels."
}
```

<sub>Module `anim` line 720</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DrawDesc"></a>

### `DrawDesc` (desc struct)

What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen through a material. Colours given as theme tokens follow the style; every value is data an agent can read and set.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `DrawShape \| null` | `null` | — | — | — | Its outline; null is its rectangle with its radius. |
| `fills` | `readonly Fill[]` | `[]` | — | — | — | Fills, bottom first. |
| `strokes` | `readonly Stroke[]` | `[]` | — | — | — | Strokes, over the fills and the children, bottom first. |
| `effects` | `readonly Effect[]` | `[]` | — | — | — | Shadows, glows, blurs and backdrops. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | Multiplies its paints (with a layer, its composite). |
| `blend` | `BlendMode` | `"normal"` | — | — | — | How its paints (or its layer) mix with what is under them. |
| `clip` | `ClipMode` | `"none"` | — | — | — | How it cuts its children. |
| `mask` | `MaskDesc \| null` | `null` | — | — | — | The mask, for clip mask. |
| `layer` | `LayerDesc \| null` | `null` | — | — | — | Its subtree drawn offscreen and composited through a material. |
| `text` | `TextEffectsDesc \| null` | `null` | — | — | — | Effects of its text. |
| `dressable` | `boolean` | `false` | — | — | — | The draw is a component's own look for the part its element names (ElementDesc.part): where the style dresses that part, each list the dressing sets (fills, strokes, effects, material, mask, blend) replaces the draw's, a rect takes the dressing's corner, and atlas art replaces the draw. The draw's shape stays, so a ring, a star or a segment keeps its geometry in every style. False: the draw is the game's and the style never touches it. |

```ogt
struct DrawDesc {
  doc "What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen through a material. Colours given as theme tokens follow the style; every value is data an agent can read and set."
  desc true
  field shape Option.DrawShape "Its outline; null is its rectangle with its radius."
  field fills List.Fill "Fills, bottom first."
  field strokes List.Stroke "Strokes, over the fills and the children, bottom first."
  field effects List.Effect "Shadows, glows, blurs and backdrops."
  field opacity f32 "Multiplies its paints (with a layer, its composite)." {
    default 1
    min 0
    max 1
  }
  field blend BlendMode "How its paints (or its layer) mix with what is under them."
  field clip ClipMode "How it cuts its children."
  field mask Option.MaskDesc "The mask, for clip mask."
  field layer Option.LayerDesc "Its subtree drawn offscreen and composited through a material."
  field text Option.TextEffectsDesc "Effects of its text."
  field dressable bool "The draw is a component's own look for the part its element names (ElementDesc.part): where the style dresses that part, each list the dressing sets (fills, strokes, effects, material, mask, blend) replaces the draw's, a rect takes the dressing's corner, and atlas art replaces the draw. The draw's shape stays, so a ring, a star or a segment keeps its geometry in every style. False: the draw is the game's and the style never touches it."
}
```

<sub>Module `ui` line 7323</sub>

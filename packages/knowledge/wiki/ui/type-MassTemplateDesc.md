<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassTemplateDesc"></a>

### `MassTemplateDesc` (desc struct)

A mass element template (docs/ui.md §1): declared once, then instanced thousands of times a frame with a few values each. No instance is laid out; its motion is the timeline, evaluated on the GPU.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MassKind` | `"number"` | — | — | — | What it draws. |
| `space` | `MassSpace` | `"world"` | — | — | — | Where instances' positions are. |
| `font` | `Font \| null` | `null` | — | — | — | Numbers and text: the font (one page); null is the UI's body font. |
| `texture` | `Texture \| null` | `null` | — | — | — | Icons: the texture its frames are in. |
| `strings` | `readonly string[]` | `[]` | — | — | — | Text: the strings an instance's value picks. |
| `frames` | `readonly number[]` | `[]` | — | — | — | Icons: uv rectangles, four numbers each (u0, v0, u1, v1). |
| `decimals` | `number` | `0` | — | — | — | Numbers: digits after the point. |
| `plus` | `boolean` | `false` | — | — | — | Numbers: a + before positive values. |
| `compact` | `boolean` | `false` | — | — | — | Numbers: 12.3k, 4.5M. |
| `barBack` | `Paint \| null` | `null` | — | — | — | Bars: the track's ink; null is near black at 85 %. |
| `barGhost` | `Paint \| null` | `null` | — | — | — | Bars: the ghost's ink; null is pale. |
| `barRadius` | `number` | `0` | ≥ 0 | `cssPx` | — | Bars: corner radius. |
| `particleShape` | `ParticleShape` | `"square"` | — | — | — | Particles: the shape. |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | Text and numbers: x is the em size; bars, icons and particles: width and height. |
| `pivot` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The element's point on its position, fractions (0.5, 1: its bottom centre). |
| `lifetime` | `number` | `0` | — | — | — | Ticks an instance lives; 0 shows pushed instances for their tick only. |
| `keys` | `readonly MassKey[]` | `[]` | — | — | — | The timeline over its life (up to 8). |
| `drift` | `number` | `0` | — | `cssPx` | — | Each instance drifts sideways up to this far over its life, by its seed. |
| `gravity` | `number` | `0` | — | `cssPx` | — | Added downward over its life, this much at its end. |
| `drag` | `boolean` | `false` | — | — | — | Instances' velocity slows to rest over the life. |
| `reference` | `number` | `0` | ≥ 0 | `m` | — | World space: the distance at which it is its size, nearer larger and farther smaller; 0 keeps its screen size. |
| `minScale` | `number` | `0.25` | ≥ 0 | — | — | World space: the smallest distance scale. |
| `maxScale` | `number` | `4` | ≥ 0 | — | — | World space: the largest distance scale. |
| `occlusion` | `WorldOcclusion` | `"none"` | — | — | — | World space: what the scene in front of an instance does. |
| `occludedAlpha` | `number` | `0.25` | 0 … 1 | — | — | Its opacity behind the scene, for occlusion fade. |
| `outline` | `Paint \| null` | `null` | — | — | — | Text and numbers: an outline's ink; null draws none. |
| `outlineWidth` | `number` | `2` | ≥ 0 | `cssPx` | — | Its width. |

```ogt
struct MassTemplateDesc {
  doc "A mass element template (docs/ui.md §1): declared once, then instanced thousands of times a frame with a few values each. No instance is laid out; its motion is the timeline, evaluated on the GPU."
  desc true
  field kind MassKind "What it draws."
  field space MassSpace "Where instances' positions are."
  field font Option.Font "Numbers and text: the font (one page); null is the UI's body font."
  field texture Option.Texture "Icons: the texture its frames are in."
  field strings List.String "Text: the strings an instance's value picks."
  field frames List.f32 "Icons: uv rectangles, four numbers each (u0, v0, u1, v1)."
  field decimals u32 "Numbers: digits after the point."
  field plus bool "Numbers: a + before positive values."
  field compact bool "Numbers: 12.3k, 4.5M."
  field barBack Option.Paint "Bars: the track's ink; null is near black at 85 %."
  field barGhost Option.Paint "Bars: the ghost's ink; null is pale."
  field barRadius f32 "Bars: corner radius." {
    unit cssPx
    min 0
  }
  field particleShape ParticleShape "Particles: the shape."
  field size Vector2 "Text and numbers: x is the em size; bars, icons and particles: width and height." {
    unit cssPx
  }
  field pivot Vector2 "The element's point on its position, fractions (0.5, 1: its bottom centre)."
  field lifetime u32 "Ticks an instance lives; 0 shows pushed instances for their tick only."
  field keys List.MassKey "The timeline over its life (up to 8)."
  field drift f32 "Each instance drifts sideways up to this far over its life, by its seed." {
    unit cssPx
  }
  field gravity f32 "Added downward over its life, this much at its end." {
    unit cssPx
  }
  field drag bool "Instances' velocity slows to rest over the life."
  field reference f32 "World space: the distance at which it is its size, nearer larger and farther smaller; 0 keeps its screen size." {
    unit m
    min 0
  }
  field minScale f32 "World space: the smallest distance scale." {
    default 0.25
    min 0
  }
  field maxScale f32 "World space: the largest distance scale." {
    default 4
    min 0
  }
  field occlusion WorldOcclusion "World space: what the scene in front of an instance does."
  field occludedAlpha f32 "Its opacity behind the scene, for occlusion fade." {
    default 0.25
    min 0
    max 1
  }
  field outline Option.Paint "Text and numbers: an outline's ink; null draws none."
  field outlineWidth f32 "Its width." {
    default 2
    unit cssPx
    min 0
  }
}
```

<sub>Module `ui` line 3772</sub>

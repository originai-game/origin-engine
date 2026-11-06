<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ShadowEffect"></a>

### `ShadowEffect` (desc struct)

A shadow: a drop shadow under the element, or an inner shadow cast inwards.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 4 }` | — | `cssPx` | — | How far it falls. |
| `blur` | `number` | `12` | ≥ 0 | `cssPx` | — | Its softness (CSS blur: the Gaussian's deviation is half of it). |
| `spread` | `number` | `0` | — | `cssPx` | — | How far it grows (or with a negative value shrinks) past the shape. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |

```ogt
struct ShadowEffect {
  doc "A shadow: a drop shadow under the element, or an inner shadow cast inwards."
  desc true
  field offset Vector2 "How far it falls." {
    default 0 4
    unit cssPx
  }
  field blur f32 "Its softness (CSS blur: the Gaussian's deviation is half of it)." {
    default 12
    min 0
    unit cssPx
  }
  field spread f32 "How far it grows (or with a negative value shrinks) past the shape." {
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
}
```

<sub>`api.ogt` line 22813</sub>

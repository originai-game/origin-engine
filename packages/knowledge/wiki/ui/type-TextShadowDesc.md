<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextShadowDesc"></a>

### `TextShadowDesc` (desc struct)

A shadow under an element's text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 2 }` | — | `cssPx` | — | How far it falls. |
| `blur` | `number` | `2` | ≥ 0 | `cssPx` | — | Its softness. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |

```ogt
struct TextShadowDesc {
  doc "A shadow under an element's text."
  desc true
  field offset Vector2 "How far it falls." {
    default 0 2
    unit cssPx
  }
  field blur f32 "Its softness." {
    default 2
    min 0
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
}
```

<sub>Module `ui` line 7283</sub>

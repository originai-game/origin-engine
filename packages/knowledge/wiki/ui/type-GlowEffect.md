<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GlowEffect"></a>

### `GlowEffect` (desc struct)

A glow: around the element, or along its inside.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `blur` | `number` | `12` | ≥ 0 | `cssPx` | — | Its softness. |
| `spread` | `number` | `0` | — | `cssPx` | — | How far it grows past the shape. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's primary. |

```ogt
struct GlowEffect {
  doc "A glow: around the element, or along its inside."
  desc true
  field blur f32 "Its softness." {
    default 12
    min 0
    unit cssPx
  }
  field spread f32 "How far it grows past the shape." {
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's primary."
}
```

<sub>Module `ui` line 2801</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassKey"></a>

### `MassKey` (desc struct)

A key of a mass element's timeline, evaluated in the shader.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `t` | `number` | `0` | 0 … 1 | `ratio` | — | Where in its life, 0–1. |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Moves it right. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Moves it down (negative rises). |
| `scale` | `number` | `1` | ≥ 0 | — | — | Its scale. |
| `alpha` | `number` | `1` | 0 … 1 | — | — | Its opacity. |
| `ease` | `Ease` | `"none"` | — | — | — | Shared GSAP names or cubic-bezier()/steps() strings. Non-opcode curves use a deduplicated 4097-sample presentation LUT; step progress is quantized before lookup. |

```ogt
struct MassKey {
  doc "A key of a mass element's timeline, evaluated in the shader."
  desc true
  field t f32 "Where in its life, 0–1." {
    unit ratio
    min 0
    max 1
  }
  field offsetX f32 "Moves it right." {
    unit cssPx
  }
  field offsetY f32 "Moves it down (negative rises)." {
    unit cssPx
  }
  field scale f32 "Its scale." {
    default 1
    min 0
  }
  field alpha f32 "Its opacity." {
    default 1
    min 0
    max 1
  }
  field ease Ease "Shared GSAP names or cubic-bezier()/steps() strings. Non-opcode curves use a deduplicated 4097-sample presentation LUT; step progress is quantized before lookup." {
    default none
  }
}
```

<sub>Module `ui` line 3488</sub>

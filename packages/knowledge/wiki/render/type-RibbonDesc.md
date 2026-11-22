<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RibbonDesc"></a>

### `RibbonDesc` (desc struct)

An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly RibbonPoint[]` | `[]` | — | — | — | The points, in order (at least two). |
| `texture` | `Texture \| null` | `null` | — | — | — | A texture along the strip; null is solid. |
| `uvTile` | `number` | `0` | ≥ 0 | `m` | — | The texture repeats every this many world units; 0 stretches it once. |
| `blend` | `Blend` | `"additive"` | — | — | — | Additive (the default: a glowing streak) or alpha blended (smoke, tyre marks). |

```ogt
struct RibbonDesc {
  doc "An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show."
  desc true
  field points List.RibbonPoint "The points, in order (at least two)."
  field texture Option.Texture "A texture along the strip; null is solid."
  field uvTile f32 "The texture repeats every this many world units; 0 stretches it once." {
    min 0
    unit m
  }
  field blend Blend "Additive (the default: a glowing streak) or alpha blended (smoke, tyre marks)." {
    default additive
  }
}
```

<sub>Module `render` line 2511</sub>

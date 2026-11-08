<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassInstance"></a>

### `MassInstance` (desc struct)

One mass element: a few values; its template does the rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | A world point, or CSS pixels (x, y) in screen space. |
| `value` | `number` | `0` | — | — | — | The number, the string or frame index, or the bar's fill 0–1. |
| `color` | `Paint \| null` | `null` | — | — | — | Its ink (text, icon tint, bar fill, particle); null is white. |
| `scale` | `number` | `1` | ≥ 0 | — | — | Multiplies its timeline's scale (a critical hit). |
| `seed` | `number` | `0` | — | — | — | Its drift's side. |
| `ghost` | `number` | `0` | — | `ratio` | — | Bars: the ghost's fill 0–1. |
| `velocity` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | Travelled over its life. |
| `spin` | `number` | `0` | — | — | — | Turns over its life. |
| `rotation` | `number` | `0` | — | `rad` | — | Its starting rotation, clockwise. |

```ogt
struct MassInstance {
  doc "One mass element: a few values; its template does the rest."
  desc true
  field position Vector3 "A world point, or CSS pixels (x, y) in screen space."
  field value f64 "The number, the string or frame index, or the bar's fill 0–1."
  field color Option.Paint "Its ink (text, icon tint, bar fill, particle); null is white."
  field scale f32 "Multiplies its timeline's scale (a critical hit)." {
    default 1
    min 0
  }
  field seed u32 "Its drift's side."
  field ghost f32 "Bars: the ghost's fill 0–1." {
    unit ratio
  }
  field velocity Vector2 "Travelled over its life." {
    unit cssPx
  }
  field spin f32 "Turns over its life."
  field rotation f32 "Its starting rotation, clockwise." {
    unit rad
  }
}
```

<sub>Module `ui` line 3474</sub>

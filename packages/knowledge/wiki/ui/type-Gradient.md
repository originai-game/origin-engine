<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Gradient"></a>

### `Gradient` (desc struct)

A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `type` | `GradientType` | `"linear"` | — | — | — | Its shape. |
| `from` | `Vector2` | `{ x: 0, y: 0 }` | — | `ratio` | — | Its start or centre, a fraction of the box. |
| `to` | `Vector2` | `{ x: 0, y: 1 }` | — | `ratio` | — | Its end, or its radii, a fraction of the box. |
| `stops` | `readonly GradientStop[]` | `[]` | — | — | — | Its stops in order (at most 8). |
| `spread` | `GradientSpread` | `"pad"` | — | — | — | Past its ends. |
| `noDither` | `boolean` | `false` | — | — | — | Draws it without the dither. |

```ogt
struct Gradient {
  doc "A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands."
  desc true
  field type GradientType "Its shape."
  field from Vector2 "Its start or centre, a fraction of the box." {
    unit ratio
  }
  field to Vector2 "Its end, or its radii, a fraction of the box." {
    default 0 1
    unit ratio
  }
  field stops List.GradientStop "Its stops in order (at most 8)."
  field spread GradientSpread "Past its ends."
  field noDither bool "Draws it without the dither."
}
```

<sub>Module `ui` line 3364</sub>

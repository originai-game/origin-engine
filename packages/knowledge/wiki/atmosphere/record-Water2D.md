<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Water2D"></a>

### `og.Water2D` (component)

Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface.

Token `og.Water2D`, registered under the schema name `og.Water2D`; its value type is `Water2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `top` | `Color` | `{ r: 0.16, g: 0.5, b: 0.72 }` | — | — | — | The body's colour at the surface. |
| `topOpacity` | `number` | `0.72` | 0 … 1 | `ratio` | — | Its opacity there. |
| `bottom` | `Color` | `{ r: 0.02, g: 0.1, b: 0.25 }` | — | — | — | The colour at the bottom. |
| `bottomOpacity` | `number` | `0.92` | 0 … 1 | `ratio` | — | Its opacity there. |
| `lip` | `Color` | `{ r: 0.8, g: 0.95, b: 1 }` | — | — | — | The lip along the surface. |
| `lipWidth` | `number` | `0.06` | ≥ 0 | `m` | tunable | How thick the lip is (0: none). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it is drawn in. |

```ogt
component Water2D {
  doc "Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface."
  field top Color "The body's colour at the surface." {
    default 0.16 0.5 0.72
  }
  field topOpacity f32 "Its opacity there." {
    default 0.72
    min 0
    max 1
    unit ratio
  }
  field bottom Color "The colour at the bottom." {
    default 0.02 0.1 0.25
  }
  field bottomOpacity f32 "Its opacity there." {
    default 0.92
    min 0
    max 1
    unit ratio
  }
  field lip Color "The lip along the surface." {
    default 0.8 0.95 1
  }
  field lipWidth f32 "How thick the lip is (0: none)." {
    default 0.06
    min 0
    unit m
    tunable true
  }
  field layer i32 "The sprite layer it is drawn in."
}
```

<sub>`api.ogt` line 11426</sub>

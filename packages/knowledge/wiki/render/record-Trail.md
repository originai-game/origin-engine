<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Trail"></a>

### `og.Trail` (component)

A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail.

Token `og.Trail`, registered under the schema name `og.Trail`; its value type is `Trail`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lifetime` | `number` | `0.4` | ≥ 0 | `s` | tunable | How long a position stays in the trail. |
| `minDistance` | `number` | `0.05` | ≥ 0 | `m` | — | A new point is kept when the head has moved this far. |
| `widthStart` | `number` | `0.15` | ≥ 0 | `m` | tunable | The width at the head. |
| `widthEnd` | `number` | `0` | ≥ 0 | `m` | — | The width at the tail. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour at the head; above 1 blooms. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha at the head. |
| `colorEnd` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour at the tail. |
| `opacityEnd` | `number` | `0` | 0 … 1 | — | — | The alpha at the tail. |
| `texture` | `Texture` | `0` | — | — | — | A texture along the strip; 0 is solid. |
| `uvTile` | `number` | `0` | ≥ 0 | `m` | — | The texture repeats every this many world units; 0 stretches it once over the length. |
| `blend` | `StripBlend` | `"additive"` | — | — | — | Additive or alpha blended. |
| `layered` | `boolean` | `false` | — | — | — | Draw it in the 2D sprite order at `layer` (a sprite's trail over a tile background) instead of with the 3D scene. |
| `layer` | `number` | `0` | — | — | — | With layered: the sprite layer it is drawn in, before that layer's sprites. |

```ogt
component Trail {
  doc "A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail."
  field lifetime f32 "How long a position stays in the trail." {
    default 0.4
    min 0
    unit s
    tunable true
  }
  field minDistance f32 "A new point is kept when the head has moved this far." {
    default 0.05
    min 0
    unit m
  }
  field widthStart f32 "The width at the head." {
    default 0.15
    min 0
    unit m
    tunable true
  }
  field widthEnd f32 "The width at the tail." {
    min 0
    unit m
  }
  field color Color "The linear colour at the head; above 1 blooms." {
    default 1 1 1
  }
  field opacity f32 "The alpha at the head." {
    default 1
    min 0
    max 1
  }
  field colorEnd Color "The linear colour at the tail." {
    default 1 1 1
  }
  field opacityEnd f32 "The alpha at the tail." {
    min 0
    max 1
  }
  field texture Texture "A texture along the strip; 0 is solid."
  field uvTile f32 "The texture repeats every this many world units; 0 stretches it once over the length." {
    min 0
    unit m
  }
  field blend StripBlend "Additive or alpha blended."
  field layered bool "Draw it in the 2D sprite order at `layer` (a sprite's trail over a tile background) instead of with the 3D scene."
  field layer i32 "With layered: the sprite layer it is drawn in, before that layer's sprites."
}
```

<sub>Module `render` line 2216</sub>

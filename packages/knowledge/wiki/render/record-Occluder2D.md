<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Occluder2D"></a>

### `og.Occluder2D` (component)

A shape that blocks 2D light (walls, crates, characters), centred on the entity's position and turned and scaled with it about z: a width by height rectangle, or the polygon `points` gives. Pixels inside it are not shadowed by it. At most 128 are drawn.

Token `og.Occluder2D`, registered under the schema name `og.Occluder2D`; its value type is `Occluder2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its width in world units (without points). |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its height in world units (without points). |
| `points` | `readonly number[]` | `[]` | — | — | a list of at most 32 `f32` (`List.f32.32`): reads a frozen copy, writes a whole array | A polygon instead of the rectangle: up to 16 vertices as x, y pairs in the entity's frame, in order around it; concave is fine. Fewer than three vertices keeps the rectangle. |
| `layers` | `number` | `4294967295` | — | — | — | The light layers whose lights it blocks (a bit per layer, as Light2D.layers). |

```ogt
component Occluder2D {
  doc "A shape that blocks 2D light (walls, crates, characters), centred on the entity's position and turned and scaled with it about z: a width by height rectangle, or the polygon `points` gives. Pixels inside it are not shadowed by it. At most 128 are drawn."
  field width f32 "Its width in world units (without points)." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its height in world units (without points)." {
    default 1
    min 0
    unit m
  }
  field points List.f32.32 "A polygon instead of the rectangle: up to 16 vertices as x, y pairs in the entity's frame, in order around it; concave is fine. Fewer than three vertices keeps the rectangle."
  field layers u32 "The light layers whose lights it blocks (a bit per layer, as Light2D.layers)." {
    default 4294967295
  }
}
```

<sub>Module `render` line 2107</sub>

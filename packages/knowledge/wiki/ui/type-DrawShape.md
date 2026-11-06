<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DrawShape"></a>

### `DrawShape` (desc struct)

An element's outline; its fills, strokes, effects and clip take it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ShapeType` | `"rect"` | — | — | — | Which outline. |
| `radii` | `[number, number, number, number] \| null` | `null` | — | `cssPx` | — | A rect's corner radii, top left, top right, bottom right, bottom left; null is the element's radius at every corner. |
| `corner` | `CornerShape` | `"round"` | — | — | — | A rect's corner shape. |
| `squircle` | `number` | `4` | 2 … 12 | — | — | A squircle corner's exponent: 2 is round, 4 the iOS look. |
| `sides` | `number` | `6` | 3 … 64 | — | — | A polygon's vertices. |
| `inner` | `number` | `1` | ≥ 0 | `ratio` | — | A star's inner vertices as a fraction of its radius; 1 (or more) is a plain polygon. |
| `rounding` | `number` | `0` | — | `cssPx` | — | A polygon's corner rounding. |
| `rotation` | `number` | `0` | — | `rad` | — | A polygon's turn, clockwise. |
| `start` | `number` | `0` | — | `rad` | — | An arc's start, clockwise from 3 o'clock. |
| `sweep` | `number` | `6.2831855` | — | `rad` | — | An arc's sweep, clockwise (a full turn or more: a ring). |
| `thickness` | `number` | `4` | — | `cssPx` | — | An arc's thickness, inwards from the circle. |
| `cap` | `LineCap` | `"butt"` | — | — | — | An arc's or a path's ends. |
| `points` | `readonly Vector2[]` | `[]` | — | `cssPx` | — | A path's points, CSS px from the element's top left. |
| `curves` | `boolean` | `false` | — | — | — | A path's points after the first come in threes: a cubic Bézier's two controls and its end. |
| `closed` | `boolean` | `false` | — | — | — | A path returns to its first point. |
| `filled` | `boolean` | `false` | — | — | — | A path is an area: its fills cover the inside of the closed outline through its points (even-odd) instead of drawing its line, and its strokes outline it, as any other shape's (an area chart, a radar's polygon, a pie's wedge). |
| `width` | `number` | `2` | ≥ 0 | `cssPx` | — | A path's line width. |
| `join` | `LineJoin` | `"miter"` | — | — | — | Where a path's segments meet. |

```ogt
struct DrawShape {
  doc "An element's outline; its fills, strokes, effects and clip take it."
  desc true
  field kind ShapeType "Which outline."
  field radii Option.Array.f32.4 "A rect's corner radii, top left, top right, bottom right, bottom left; null is the element's radius at every corner." {
    unit cssPx
  }
  field corner CornerShape "A rect's corner shape."
  field squircle f32 "A squircle corner's exponent: 2 is round, 4 the iOS look." {
    default 4
    min 2
    max 12
  }
  field sides u32 "A polygon's vertices." {
    default 6
    min 3
    max 64
  }
  field inner f32 "A star's inner vertices as a fraction of its radius; 1 (or more) is a plain polygon." {
    default 1
    min 0
    unit ratio
  }
  field rounding f32 "A polygon's corner rounding." {
    unit cssPx
  }
  field rotation f32 "A polygon's turn, clockwise." {
    unit rad
  }
  field start f32 "An arc's start, clockwise from 3 o'clock." {
    unit rad
  }
  field sweep f32 "An arc's sweep, clockwise (a full turn or more: a ring)." {
    default 6.2831855
    unit rad
  }
  field thickness f32 "An arc's thickness, inwards from the circle." {
    default 4
    unit cssPx
  }
  field cap LineCap "An arc's or a path's ends."
  field points List.Vector2 "A path's points, CSS px from the element's top left." {
    unit cssPx
  }
  field curves bool "A path's points after the first come in threes: a cubic Bézier's two controls and its end."
  field closed bool "A path returns to its first point."
  field filled bool "A path is an area: its fills cover the inside of the closed outline through its points (even-odd) instead of drawing its line, and its strokes outline it, as any other shape's (an area chart, a radar's polygon, a pie's wedge)."
  field width f32 "A path's line width." {
    default 2
    min 0
    unit cssPx
  }
  field join LineJoin "Where a path's segments meet."
}
```

<sub>Module `ui` line 7069</sub>

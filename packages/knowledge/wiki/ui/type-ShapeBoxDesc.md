<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ShapeBoxDesc"></a>

### `ShapeBoxDesc` (desc struct)

A decorative shape: its outline, fill, stroke and glow, in the style's colours.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ShapePreset` | `"rect"` | — | — | — | Its outline. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 24. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null is its width's. |
| `fill` | `ThemeColor \| null` | `null` | — | — | — | Its fill's colour; null is primary. |
| `fillTo` | `ThemeColor \| null` | `null` | — | — | — | A second colour: a linear gradient from fill to it at angle. |
| `angle` | `number` | `0` | — | `deg` | — | The gradient's direction, clockwise from the top (0 runs down). |
| `opacity` | `number` | `1` | 0 … 1 | — | — | Its opacity. |
| `stroke` | `ThemeColor \| null` | `null` | — | — | — | Its outline's colour; null for none. |
| `strokeWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | Its outline's width; 0 is the style's hairline. |
| `glow` | `ThemeColor \| null` | `null` | — | — | — | An outer glow of this colour; null for none. |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | A rect's corner radius. |
| `sides` | `number` | `5` | ≥ 3 | — | — | A polygon's or a star's corners. |
| `thickness` | `number` | `2` | ≥ 0 | `cssPx` | — | A ring's or a line's thickness. |
| `rotation` | `number` | `0` | — | `deg` | — | Its turn, clockwise. |

```ogt
struct ShapeBoxDesc {
  doc "A decorative shape: its outline, fill, stroke and glow, in the style's colours."
  desc true
  field kind ShapePreset "Its outline."
  field width Option.Size "Its width; null is 24."
  field height Option.Size "Its height; null is its width's."
  field fill Option.ThemeColor "Its fill's colour; null is primary."
  field fillTo Option.ThemeColor "A second colour: a linear gradient from fill to it at angle."
  field angle f32 "The gradient's direction, clockwise from the top (0 runs down)." {
    unit deg
  }
  field opacity f32 "Its opacity." {
    default 1
    min 0
    max 1
  }
  field stroke Option.ThemeColor "Its outline's colour; null for none."
  field strokeWidth f32 "Its outline's width; 0 is the style's hairline." {
    unit cssPx
    min 0
  }
  field glow Option.ThemeColor "An outer glow of this colour; null for none."
  field radius f32 "A rect's corner radius." {
    unit cssPx
    min 0
  }
  field sides u32 "A polygon's or a star's corners." {
    default 5
    min 3
  }
  field thickness f32 "A ring's or a line's thickness." {
    default 2
    unit cssPx
    min 0
  }
  field rotation f32 "Its turn, clockwise." {
    unit deg
  }
}
```

<sub>Module `ui` line 3183</sub>

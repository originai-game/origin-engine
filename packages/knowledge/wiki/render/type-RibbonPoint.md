<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RibbonPoint"></a>

### `RibbonPoint` (desc struct)

One point of a ribbon.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `width` | `number` | `0.1` | ≥ 0 | `m` | — | The width there. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour there. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha there. |

```ogt
struct RibbonPoint {
  doc "One point of a ribbon."
  desc true
  field position Vector3 "Where it is." {
    unit m
  }
  field width f32 "The width there." {
    default 0.1
    min 0
    unit m
  }
  field color Color "The linear colour there." {
    default 1 1 1
  }
  field opacity f32 "The alpha there." {
    default 1
    min 0
    max 1
  }
}
```

<sub>Module `render` line 2123</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-OverheadBarsDesc"></a>

### `OverheadBarsDesc` (desc struct)

Overhead bars over many units.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bars` | `readonly OverheadBar[]` | `[]` | — | — | — | The bars this tick. |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | A bar's width; 0 is 56. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | A bar's height; 0 is 6. |
| `reference` | `number` | `12` | ≥ 0 | `m` | — | The distance at which a bar is its size (nearer larger, farther smaller); 0 keeps the screen size. |
| `hideFull` | `boolean` | `false` | — | — | — | A bar at full health is not drawn (an RTS's default). |

```ogt
struct OverheadBarsDesc {
  doc "Overhead bars over many units."
  desc true
  field bars List.OverheadBar "The bars this tick."
  field width f32 "A bar's width; 0 is 56." {
    min 0
    unit cssPx
  }
  field height f32 "A bar's height; 0 is 6." {
    min 0
    unit cssPx
  }
  field reference f32 "The distance at which a bar is its size (nearer larger, farther smaller); 0 keeps the screen size." {
    default 12
    unit m
    min 0
  }
  field hideFull bool "A bar at full health is not drawn (an RTS's default)."
}
```

<sub>Module `ui` line 6347</sub>

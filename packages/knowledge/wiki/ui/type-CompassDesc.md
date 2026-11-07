<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CompassDesc"></a>

### `CompassDesc` (desc struct)

A compass strip.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `heading` | `number` | `0` | — | `rad` | — | The player's heading, clockwise from north. |
| `fov` | `number` | `0` | ≥ 0 | `rad` | — | How much of the circle the strip spans; 0 is 180°. |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | Its width; 0 is 560 (a third of the surface on a phone). |
| `pois` | `readonly CompassPoi[]` | `[]` | — | — | — | Its points. |
| `points` | `readonly string[]` | `[]` | — | — | — | The eight point names from north clockwise (N, NE, E …); empty is English. |
| `units` | `string` | `"m"` | — | — | — | The distance's unit. |
| `anchor` | `Anchor \| null` | `"top"` | — | — | — | Opens its own anchored group there; null builds it in place. |

```ogt
struct CompassDesc {
  doc "A compass strip."
  desc true
  field heading f32 "The player's heading, clockwise from north." {
    unit rad
  }
  field fov f32 "How much of the circle the strip spans; 0 is 180°." {
    min 0
    unit rad
  }
  field width f32 "Its width; 0 is 560 (a third of the surface on a phone)." {
    min 0
    unit cssPx
  }
  field pois List.CompassPoi "Its points."
  field points List.String "The eight point names from north clockwise (N, NE, E …); empty is English."
  field units String "The distance's unit." {
    default "m"
  }
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place." {
    default top
  }
}
```

<sub>Module `ui` line 6272</sub>

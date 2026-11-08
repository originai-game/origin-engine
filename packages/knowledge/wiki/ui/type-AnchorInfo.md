<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AnchorInfo"></a>

### `AnchorInfo` (struct)

Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `inView` | `boolean` | `false` | — | — | — | Its point is in front of the camera and within maxDistance (or clamped to the edge). |
| `onScreen` | `boolean` | `false` | — | — | — | Its point projects inside the screen. |
| `clamped` | `boolean` | `false` | — | — | — | Held on the screen's edge. |
| `lod` | `number` | `0` | — | — | — | Its level of detail. |
| `distance` | `number` | `0` | — | `m` | — | From the camera. |
| `x` | `number` | `0` | — | `cssPx` | — | Where its pivot was drawn. |
| `y` | `number` | `0` | — | `cssPx` | — | Where its pivot was drawn. |
| `angle` | `number` | `0` | — | `rad` | — | Toward its point from the screen's centre, clockwise from +x (the edge arrow's). |
| `scale` | `number` | `0` | — | — | — | The scale it was drawn at. |
| `alpha` | `number` | `0` | — | `ratio` | — | Its eased opacity (declutter, distance). |

```ogt
struct AnchorInfo {
  doc "Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it."
  field inView bool "Its point is in front of the camera and within maxDistance (or clamped to the edge)."
  field onScreen bool "Its point projects inside the screen."
  field clamped bool "Held on the screen's edge."
  field lod u32 "Its level of detail."
  field distance f32 "From the camera." {
    unit m
  }
  field x f32 "Where its pivot was drawn." {
    unit cssPx
  }
  field y f32 "Where its pivot was drawn." {
    unit cssPx
  }
  field angle f32 "Toward its point from the screen's centre, clockwise from +x (the edge arrow's)." {
    unit rad
  }
  field scale f32 "The scale it was drawn at."
  field alpha f32 "Its eased opacity (declutter, distance)." {
    unit ratio
  }
}
```

<sub>Module `ui` line 7395</sub>

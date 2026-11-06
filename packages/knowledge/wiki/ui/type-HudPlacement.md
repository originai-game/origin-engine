<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HudPlacement"></a>

### `HudPlacement` (struct)

Where the player put a customisable element: moved from its laid-out place, scaled about its centre and faded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Moved right. |
| `y` | `number` | `0` | — | `cssPx` | — | Moved down. |
| `scale` | `number` | `0` | — | `ratio` | — | Its scale (1: as laid out). |
| `opacity` | `number` | `0` | — | `ratio` | — | Its opacity (1: opaque). |

```ogt
struct HudPlacement {
  doc "Where the player put a customisable element: moved from its laid-out place, scaled about its centre and faded."
  field x f32 "Moved right." {
    unit cssPx
  }
  field y f32 "Moved down." {
    unit cssPx
  }
  field scale f32 "Its scale (1: as laid out)." {
    unit ratio
  }
  field opacity f32 "Its opacity (1: opaque)." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 17394</sub>

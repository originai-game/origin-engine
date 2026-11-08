<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TrackRange"></a>

### `TrackRange` (struct)

A track sized by its content, within bounds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | `cssPx` | — | The smallest it gets. |
| `max` | `number` | `0` | — | `cssPx` | — | The largest it gets. |

```ogt
struct TrackRange {
  doc "A track sized by its content, within bounds."
  field min f32 "The smallest it gets." {
    unit cssPx
  }
  field max f32 "The largest it gets." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 4440</sub>

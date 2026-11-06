<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SizeRange"></a>

### `SizeRange` (struct)

The bounds of a fit or grow size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | `cssPx` | optional | The smallest size. |
| `max` | `number` | `0` | — | `cssPx` | optional | The largest size; absent is unbounded. |

```ogt
struct SizeRange {
  doc "The bounds of a fit or grow size."
  field min f32 "The smallest size." {
    optional true
    unit cssPx
  }
  field max f32 "The largest size; absent is unbounded." {
    optional true
    unit cssPx
  }
}
```

<sub>`api.ogt` line 15700</sub>

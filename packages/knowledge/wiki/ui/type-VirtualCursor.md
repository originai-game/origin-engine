<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-VirtualCursor"></a>

### `VirtualCursor` (struct)

Where a pad's virtual cursor is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its x, from the surface's left. |
| `y` | `number` | `0` | — | `cssPx` | — | Its y, from the surface's top. |
| `shown` | `boolean` | `false` | — | — | — | It is shown: the stick moved it since the real pointer last moved. |

```ogt
struct VirtualCursor {
  doc "Where a pad's virtual cursor is."
  field x f32 "Its x, from the surface's left." {
    unit cssPx
  }
  field y f32 "Its y, from the surface's top." {
    unit cssPx
  }
  field shown bool "It is shown: the stick moved it since the real pointer last moved."
}
```

<sub>Module `ui` line 604</sub>

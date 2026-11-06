<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PanelInfo"></a>

### `PanelInfo` (struct)

What the pointer does on a world panel this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pointed` | `boolean` | `false` | — | — | — | The ray from the pointer meets the panel's quad (and no screen element is under it): its controls take the press. |
| `x` | `number` | `0` | — | `cssPx` | — | Where on the panel, from its top left. |
| `y` | `number` | `0` | — | `cssPx` | — | Where on the panel, from its top left. |
| `distance` | `number` | `0` | — | `m` | — | Along the ray to the hit. |

```ogt
struct PanelInfo {
  doc "What the pointer does on a world panel this tick."
  field pointed bool "The ray from the pointer meets the panel's quad (and no screen element is under it): its controls take the press."
  field x f32 "Where on the panel, from its top left." {
    unit cssPx
  }
  field y f32 "Where on the panel, from its top left." {
    unit cssPx
  }
  field distance f32 "Along the ray to the hit." {
    unit m
  }
}
```

<sub>`api.ogt` line 23296</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SnapPoint"></a>

### `SnapPoint` (struct)

An offset a released drag can settle on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its x offset. |
| `y` | `number` | `0` | — | `cssPx` | — | Its y offset. |

```ogt
struct SnapPoint {
  doc "An offset a released drag can settle on."
  field x f32 "Its x offset." {
    unit cssPx
  }
  field y f32 "Its y offset." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 4622</sub>

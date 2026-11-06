<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Presence"></a>

### `Presence` (struct)

Where a transition or dialog is this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | Build its content: shown, entering or leaving. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 gone, 1 settled. |
| `entering` | `boolean` | `false` | — | — | — | Moving in. |
| `leaving` | `boolean` | `false` | — | — | — | Moving out. |

```ogt
struct Presence {
  doc "Where a transition or dialog is this tick."
  field visible bool "Build its content: shown, entering or leaving."
  field progress f32 "0 gone, 1 settled." {
    unit ratio
  }
  field entering bool "Moving in."
  field leaving bool "Moving out."
}
```

<sub>`api.ogt` line 16577</sub>

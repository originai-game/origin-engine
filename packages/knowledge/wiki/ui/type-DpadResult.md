<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DpadResult"></a>

### `DpadResult` (struct)

Which arms of a d-pad are held this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `up` | `boolean` | `false` | — | — | — | Up is held. |
| `down` | `boolean` | `false` | — | — | — | Down is held. |
| `left` | `boolean` | `false` | — | — | — | Left is held. |
| `right` | `boolean` | `false` | — | — | — | Right is held. |

```ogt
struct DpadResult {
  doc "Which arms of a d-pad are held this tick."
  field up bool "Up is held."
  field down bool "Down is held."
  field left bool "Left is held."
  field right bool "Right is held."
}
```

<sub>Module `ui` line 1116</sub>

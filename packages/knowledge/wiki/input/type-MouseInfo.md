<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-MouseInfo"></a>

### `MouseInfo` (struct)

The mouse as the host reads it (og.input.mouse).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `raw` | `boolean` | `false` | — | — | — | Raw input is asked for (MouseDesc.raw). |
| `rawActive` | `boolean` | `false` | — | — | — | The lock now delivers raw movement: the browser granted unadjustedMovement. False while unlocked. |
| `rawUnsupported` | `boolean` | `false` | — | — | — | The browser refused unadjustedMovement (Firefox, Safari, Linux): its lock moves with the OS's acceleration. A settings screen shows the Raw input switch as unavailable then. |

```ogt
struct MouseInfo {
  doc "The mouse as the host reads it (og.input.mouse)."
  field raw bool "Raw input is asked for (MouseDesc.raw)."
  field rawActive bool "The lock now delivers raw movement: the browser granted unadjustedMovement. False while unlocked."
  field rawUnsupported bool "The browser refused unadjustedMovement (Firefox, Safari, Linux): its lock moves with the OS's acceleration. A settings screen shows the Raw input switch as unavailable then."
}
```

<sub>Module `input` line 273</sub>

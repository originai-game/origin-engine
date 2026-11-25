<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-LayersDump"></a>

### `LayersDump` (struct)

The UI layer stack alone (docs/ui.md §6).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `layers` | `unknown` | `null` | — | — | — | The layer stack; empty without a UI module. |

```ogt
struct LayersDump {
  doc "The UI layer stack alone (docs/ui.md §6)."
  field tick u32 "The tick."
  field layers Json "The layer stack; empty without a UI module."
}
```

<sub>Module `probe` line 598</sub>

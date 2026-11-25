<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DrawnDump"></a>

### `DrawnDump` (struct)

What the main camera's cull kept of the last frame drawn (dump_drawn, as Visibility): every entity with something to draw, in entity order.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `drawn` | `number` | `0` | — | — | — | Entities drawn (drawn or hlod). |
| `notDrawn` | `number` | `0` | — | — | — | Entities not drawn (untracked ones are neither). |
| `entities` | `readonly DrawnEntity[]` | `[]` | — | — | — | The entities. |

```ogt
struct DrawnDump {
  doc "What the main camera's cull kept of the last frame drawn (dump_drawn, as Visibility): every entity with something to draw, in entity order."
  field tick u32 "The tick."
  field drawn u32 "Entities drawn (drawn or hlod)."
  field notDrawn u32 "Entities not drawn (untracked ones are neither)."
  field entities List.DrawnEntity "The entities."
}
```

<sub>Module `probe` line 517</sub>

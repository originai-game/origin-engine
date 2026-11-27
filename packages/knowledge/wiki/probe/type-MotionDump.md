<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-MotionDump"></a>

### `MotionDump` (struct)

The UI motion alone (docs/ui.md §6): each element in a visual state or animating, with its states, what is moving, what plays and whether it is leaving.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `motion` | `unknown` | `null` | — | — | — | The moving elements; empty without a UI module. |

```ogt
struct MotionDump {
  doc "The UI motion alone (docs/ui.md §6): each element in a visual state or animating, with its states, what is moving, what plays and whether it is leaving."
  field tick u32 "The tick."
  field motion Json "The moving elements; empty without a UI module."
}
```

<sub>Module `probe` line 603</sub>

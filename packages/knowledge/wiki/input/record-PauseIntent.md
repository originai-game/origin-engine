<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-PauseIntent"></a>

### `og.PauseIntent` (event)

An engine gameplay intent: apply scale at the next tick boundary. UI pause layers send this through the same recorded intent path as game controls.

Token `og.PauseIntent`, registered under the schema name `og.PauseIntent`; its value type is `PauseIntent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scale` | `number` | `0` | 0 … 10 | — | — | Simulation time scale; zero pauses. |

```ogt
event PauseIntent {
  doc "An engine gameplay intent: apply scale at the next tick boundary. UI pause layers send this through the same recorded intent path as game controls."
  field scale f32 "Simulation time scale; zero pauses." {
    min 0
    max 10
  }
}
```

<sub>Module `input` line 40</sub>

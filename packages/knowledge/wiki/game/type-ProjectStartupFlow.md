<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ProjectStartupFlow"></a>

### `ProjectStartupFlow` (struct)

The dialogue started before GameDesc.start; no source code or pack metadata.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue's physical cooked ID, resolved by og project from its checked project ref. |
| `screen` | `ProjectFlowScreen` | `"engine"` | — | — | optional | Who draws the flow; omitted means engine. |

```ogt
struct ProjectStartupFlow {
  doc "The dialogue started before GameDesc.start; no source code or pack metadata."
  field dialogue String "The dialogue's physical cooked ID, resolved by og project from its checked project ref."
  field screen ProjectFlowScreen "Who draws the flow; omitted means engine." {
    optional true
  }
}
```

<sub>Module `game` line 18</sub>

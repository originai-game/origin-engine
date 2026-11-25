<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowCheckpoint"></a>

### `FlowCheckpoint` (struct)

A lightweight branch-rewind checkpoint: the dialogue, node and story variables (og.flow.checkpoint / og.flow.jump).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue name. |
| `node` | `string` | `""` | — | — | — | The flow node to resume at. |
| `vars` | `unknown` | `null` | — | — | — | Snapshot of story variables at this node. |

```ogt
struct FlowCheckpoint {
  doc "A lightweight branch-rewind checkpoint: the dialogue, node and story variables (og.flow.checkpoint / og.flow.jump)."
  field dialogue String "The dialogue name."
  field node String "The flow node to resume at."
  field vars Json "Snapshot of story variables at this node."
}
```

<sub>Module `flow` line 222</sub>

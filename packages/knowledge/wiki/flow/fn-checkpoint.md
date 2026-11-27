<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-checkpoint"></a>

### `og.flow.checkpoint`

Capture a lightweight branch-rewind checkpoint: the current dialogue, flow node, and story variables.

```ts
og.flow.checkpoint(world: World, flow: Entity): Result<FlowCheckpoint>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `Result<FlowCheckpoint>`: Refused for an entity that is no flow or has no active node.

Binding symbol `og_flow_checkpoint` · publish builds: kept

```ogt
fn checkpoint {
  doc "Capture a lightweight branch-rewind checkpoint: the current dialogue, flow node, and story variables."
  param world World "The world."
  param flow Entity "The flow."
  returns Result.FlowCheckpoint "Refused for an entity that is no flow or has no active node."
}
```

<sub>Module `flow` line 420</sub>

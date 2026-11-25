<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-jump"></a>

### `og.flow.jump`

Jump a running flow directly to another flow node, optionally applying story variables from a FlowCheckpoint. Systems may call it.

```ts
og.flow.jump(world: World, flow: Entity, desc?: FlowJumpDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `desc` | `FlowJumpDesc` | yes | The target node and optional story variables. |

**Returns** `Result<void>`: Refused for an entity that is no flow or an unknown node.

Binding symbol `og_flow_jump` · publish builds: kept

```ogt
fn jump {
  doc "Jump a running flow directly to another flow node, optionally applying story variables from a FlowCheckpoint. Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param desc FlowJumpDesc "The target node and optional story variables."
  returns Result.Void "Refused for an entity that is no flow or an unknown node."
}
```

<sub>Module `flow` line 428</sub>

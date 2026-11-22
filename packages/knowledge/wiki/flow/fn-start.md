<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-start"></a>

### `og.flow.start`

Start a flow: its decks, and its conversation held at the first flow node, whose segment plays from the next tick. Outside a system (setup or a command).

```ts
og.flow.start(world: World, desc?: FlowStartDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FlowStartDesc` | yes | The dialogue, and optionally the node, the decks and whether it starts paused. |

**Returns** `Result<Entity>`: The flow's entity (og.Flow), or why not: a dialogue og.flow.load has not prepared, an unknown node, or inside a system.

Binding symbol `og_flow_start` · publish builds: kept

```ogt
fn start {
  doc "Start a flow: its decks, and its conversation held at the first flow node, whose segment plays from the next tick. Outside a system (setup or a command)."
  param world World "The world."
  param desc FlowStartDesc "The dialogue, and optionally the node, the decks and whether it starts paused."
  returns Result.Entity "The flow's entity (og.Flow), or why not: a dialogue og.flow.load has not prepared, an unknown node, or inside a system."
}
```

<sub>Module `flow` line 212</sub>

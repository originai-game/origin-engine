<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-history"></a>

### `og.flow.history`

Recent captions and player choices recorded by this flow (backlog).

```ts
og.flow.history(world: World, flow: Entity): readonly FlowTranscriptLine[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `readonly FlowTranscriptLine[]`: Empty for an entity that is no flow.

Binding symbol `og_flow_history` · publish builds: kept

```ogt
fn history {
  doc "Recent captions and player choices recorded by this flow (backlog)."
  param world World "The world."
  param flow Entity "The flow."
  returns List.FlowTranscriptLine "Empty for an entity that is no flow."
}
```

<sub>Module `flow` line 414</sub>

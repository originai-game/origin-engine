<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-view"></a>

### `og.flow.view`

What a flow shows now: its picture, segment time, choices and window.

```ts
og.flow.view(world: World, flow: Entity): FlowView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `FlowView | null`: Null for an entity that is no flow.

Binding symbol `og_flow_view` · publish builds: kept

```ogt
fn view {
  doc "What a flow shows now: its picture, segment time, choices and window."
  param world World "The world."
  param flow Entity "The flow."
  returns Option.FlowView "Null for an entity that is no flow."
}
```

<sub>Module `flow` line 226</sub>

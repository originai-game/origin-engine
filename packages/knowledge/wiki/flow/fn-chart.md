<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-chart"></a>

### `og.flow.chart`

A prepared dialogue's flow nodes and where each may lead, laid out for a flowchart.

```ts
og.flow.chart(world: World, dialogue: string): Result<FlowChart>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `dialogue` | `string` | no | The dialogue (StoryAsset.name), prepared by og.flow.load. |

**Returns** `Result<FlowChart>`: Refused for a dialogue og.flow.load has not prepared.

Binding symbol `og_flow_chart` · publish builds: kept

```ogt
fn chart {
  doc "A prepared dialogue's flow nodes and where each may lead, laid out for a flowchart."
  param world World "The world."
  param dialogue String "The dialogue (StoryAsset.name), prepared by og.flow.load."
  returns Result.FlowChart "Refused for a dialogue og.flow.load has not prepared."
}
```

<sub>Module `flow` line 443</sub>

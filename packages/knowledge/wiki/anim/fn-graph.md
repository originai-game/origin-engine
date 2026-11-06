<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-graph"></a>

### `og.anim.graph`

Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`.

```ts
og.anim.graph(world: World, desc?: GraphDesc): Result<GraphAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the graph. |
| `desc` | `GraphDesc` | yes | The text and the clips it may name. |

**Returns** `Result<GraphAsset>`: Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph.

Binding symbol `og_anim_graph` · publish builds: kept

```ogt
fn graph {
  doc "Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`."
  param world World "The world whose animation owns the graph."
  param desc GraphDesc "The text and the clips it may name."
  returns Result.GraphAsset "Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph."
}
```

<sub>Module `anim` line 577</sub>

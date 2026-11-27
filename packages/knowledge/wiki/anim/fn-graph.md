<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-graph"></a>

### `og.anim.graph`

Register a typed blend-tree graph for AnimationGraph.asset. Clip motions resolve by name against clips.

```ts
og.anim.graph(world: World, desc?: GraphDesc): Result<GraphAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the graph. |
| `desc` | `GraphDesc` | yes | Named motions, layers, parameters and the clips they may name. |

**Returns** `Result<GraphAsset>`: Refused for unknown names, cycles, invalid numbers or clip names clips lacks.

Binding symbol `og_anim_graph` · publish builds: kept

```ogt
fn graph {
  doc "Register a typed blend-tree graph for AnimationGraph.asset. Clip motions resolve by name against clips."
  param world World "The world whose animation owns the graph."
  param desc GraphDesc "Named motions, layers, parameters and the clips they may name."
  returns Result.GraphAsset "Refused for unknown names, cycles, invalid numbers or clip names clips lacks."
}
```

<sub>Module `anim` line 800</sub>

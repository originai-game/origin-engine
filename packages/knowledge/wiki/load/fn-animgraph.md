<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-animgraph"></a>

### `og.load.animgraph`

Load a cooked type animgraph asset. Its bind lines map graph names to model clip names; the cook checks references and graph structure. Sprite graphs use sprite true and resolve sheet/rig clip names at playback. Loading the same asset again returns the same handle.

```ts
og.load.animgraph(world: World, path: string): Result<GraphAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the graph. |
| `path` | `string` | no | The animgraph asset id under assets/. |

**Returns** `Result<GraphAsset>`: Refused for an uncooked or unavailable path, invalid graph or missing bound animation clip.

Binding symbol `og_load_animgraph` · publish builds: kept

```ogt
fn animgraph {
  doc "Load a cooked type animgraph asset. Its bind lines map graph names to model clip names; the cook checks references and graph structure. Sprite graphs use sprite true and resolve sheet/rig clip names at playback. Loading the same asset again returns the same handle."
  param world World "The world whose animation owns the graph."
  param path String "The animgraph asset id under assets/."
  returns Result.GraphAsset "Refused for an uncooked or unavailable path, invalid graph or missing bound animation clip."
}
```

<sub>Module `load` line 506</sub>

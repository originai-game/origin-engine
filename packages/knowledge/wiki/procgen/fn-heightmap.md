<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-heightmap"></a>

### `og.procgen.heightmap`

Generate a deterministic eroded terrain heightmap and optional biome control map as native TypedArrays (hand directly to og.terrain.create).

Also known as: generate terrain, hydraulic erosion, thermal erosion, procedural heightmap, biome map

```ts
og.procgen.heightmap(desc?: HeightmapDesc): Result<HeightmapResult>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `HeightmapDesc` | yes | Size, noise, erosion, river and biome rules. |

**Returns** `Result<HeightmapResult>`: Refused for invalid dimensions or non-finite parameters.

Binding symbol `og_procgen_heightmap` · publish builds: kept

```ogt
fn heightmap {
  doc "Generate a deterministic eroded terrain heightmap and optional biome control map as native TypedArrays (hand directly to og.terrain.create)."
  aliases "generate terrain, hydraulic erosion, thermal erosion, procedural heightmap, biome map"
  param desc HeightmapDesc "Size, noise, erosion, river and biome rules."
  returns Result.HeightmapResult "Refused for invalid dimensions or non-finite parameters."
}
```

<sub>Module `procgen` line 606</sub>

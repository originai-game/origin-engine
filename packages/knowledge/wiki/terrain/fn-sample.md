<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-sample"></a>

### `og.terrain.sample`

The surface under a world point: its height, normal and slope. Each height-field cell splits along the diagonal whose corner heights differ less; ties keep the (-x, -z) to (+x, +z) diagonal. Sampling, rendering at the finest level and the native height-field collider use this same rule. Callable inside systems (it reads the terrain, not the world).

```ts
og.terrain.sample(world: World, desc?: TerrainSampleDesc): Result<TerrainSample>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TerrainSampleDesc` | yes | The terrain, where it stands and the point. |

**Returns** `Result<TerrainSample>`: Refused for an unknown terrain.

Binding symbol `og_terrain_sample` · publish builds: kept

```ogt
fn sample {
  doc "The surface under a world point: its height, normal and slope. Each height-field cell splits along the diagonal whose corner heights differ less; ties keep the (-x, -z) to (+x, +z) diagonal. Sampling, rendering at the finest level and the native height-field collider use this same rule. Callable inside systems (it reads the terrain, not the world)."
  param world World "The world."
  param desc TerrainSampleDesc "The terrain, where it stands and the point."
  returns Result.TerrainSample "Refused for an unknown terrain."
}
```

<sub>`api.ogt` line 25303</sub>

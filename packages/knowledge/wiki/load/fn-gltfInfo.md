<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-gltfInfo"></a>

### `og.load.gltfInfo`

What a ready glTF holds.

```ts
og.load.gltfInfo(world: World, asset: Asset): Result<GltfInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A ready glTF handle. |

**Returns** `Result<GltfInfo>`: Refused for a pending, failed or non-glTF asset.

Binding symbol `og_load_gltf_info` · publish builds: kept

```ogt
fn gltfInfo {
  doc "What a ready glTF holds."
  param world World "The world that owns the asset."
  param asset Asset "A ready glTF handle."
  returns Result.GltfInfo "Refused for a pending, failed or non-glTF asset."
}
```

<sub>`api.ogt` line 8257</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-bundleInfo"></a>

### `og.load.bundleInfo`

What an open bundle holds.

```ts
og.load.bundleInfo(world: World, asset: Asset): Result<BundleInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`. |

**Returns** `Result<BundleInfo>`: Refused for a pending, failed or non-bundle asset.

Binding symbol `og_load_bundle_info` · publish builds: kept

```ogt
fn bundleInfo {
  doc "What an open bundle holds."
  param world World "The world that owns the asset."
  param asset Asset "A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`."
  returns Result.BundleInfo "Refused for a pending, failed or non-bundle asset."
}
```

<sub>Module `load` line 193</sub>

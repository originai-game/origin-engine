<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-imageInfo"></a>

### `og.load.imageInfo`

A ready uncooked image's texture and size.

```ts
og.load.imageInfo(world: World, asset: Asset): Result<ImageInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.imageData answered. |

**Returns** `Result<ImageInfo>`: Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset.

Binding symbol `og_load_image_info` · publish builds: kept

```ogt
fn imageInfo {
  doc "A ready uncooked image's texture and size."
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.imageData answered."
  returns Result.ImageInfo "Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset."
}
```

<sub>Module `load` line 568</sub>

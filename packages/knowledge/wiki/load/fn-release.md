<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-release"></a>

### `og.load.release`

Free an uncooked asset (og.load.imageData, og.load.gltfData, og.load.environmentData): a pending load is dropped (it never completes, so it is never journaled), and a ready one's textures and meshes are emptied under their handles. An og.Model of a released model draws nothing; a sprite or material of a released texture draws one clear texel; an Environment naming a released map is lit by the sky. The handle then answers failed, naming the release. Releasing again does nothing.

See `loading §10`.

```ts
og.load.release(world: World, asset: Asset): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.imageData, og.load.gltfData or og.load.environmentData answered. |

**Returns** `Result<void>`: Refused for a cooked asset, which stays with its bundle (a streamed one with its group).

Binding symbol `og_load_release` · publish builds: kept

```ogt
fn release {
  doc "Free an uncooked asset (og.load.imageData, og.load.gltfData, og.load.environmentData): a pending load is dropped (it never completes, so it is never journaled), and a ready one's textures and meshes are emptied under their handles. An og.Model of a released model draws nothing; a sprite or material of a released texture draws one clear texel; an Environment naming a released map is lit by the sky. The handle then answers failed, naming the release. Releasing again does nothing."
  guide "loading §10"
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.imageData, og.load.gltfData or og.load.environmentData answered."
  returns Result.Void "Refused for a cooked asset, which stays with its bundle (a streamed one with its group)."
}
```

<sub>Module `load` line 592</sub>

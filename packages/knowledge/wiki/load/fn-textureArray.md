<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-textureArray"></a>

### `og.load.textureArray`

Load a cooked texture array (an `.ogt` of `type texture-array`: a `kind` and 1 to 256 `layer` images of one size) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture, `<path>#layer<n>`. Loading the same path again answers the same array.

Sprites (SpriteDesc.array, og.Sprite.array) and 2D particle emitters draw from it, one draw per array. In a streaming group it loads once the group is resident (refused before, naming the group), draws nothing while the group is away and returns under the same handle.

See `loading §8`.

```ts
og.load.textureArray(world: World, path: string): Result<TextureArrayInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the array. |
| `path` | `string` | no | The array's asset id, such as `tiles/ground.ogt`. |

**Returns** `Result<TextureArrayInfo>`: Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree.

Binding symbol `og_load_texture_array` · publish builds: kept

```ogt
fn textureArray {
  doc "Load a cooked texture array (an `.ogt` of `type texture-array`: a `kind` and 1 to 256 `layer` images of one size) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture, `<path>#layer<n>`. Loading the same path again answers the same array."
  doc "Sprites (SpriteDesc.array, og.Sprite.array) and 2D particle emitters draw from it, one draw per array. In a streaming group it loads once the group is resident (refused before, naming the group), draws nothing while the group is away and returns under the same handle."
  guide "loading §8"
  param world World "The world whose renderer owns the array."
  param path String "The array's asset id, such as `tiles/ground.ogt`."
  returns Result.TextureArrayInfo "Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree."
}
```

<sub>Module `load` line 328</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-textureArray"></a>

### `og.load.textureArray`

Load a cooked TS file['texture-array'] declaration as one 2D-array texture: 1..256 equally sized images, in declaration order, encoded for their kind. Each layer is also a texture, `<path>#layer<n>`. Repeated loads return the same array.

Sprites (SpriteDesc.array, og.Sprite.array) and 2D particle emitters draw from it, one draw per array. In a streaming group it loads once the group is resident (refused before, naming the group), draws nothing while the group is away and returns under the same handle.

```ts
og.load.textureArray(world: World, path: string): Result<TextureArrayInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the array. |
| `path` | `string` | no | The array's canonical asset id, such as `base/tiles/ground`. |

**Returns** `Result<TextureArrayInfo>`: Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree.

Binding symbol `og_load_texture_array` · publish builds: kept

```ogt
fn textureArray {
  doc "Load a cooked TS file['texture-array'] declaration as one 2D-array texture: 1..256 equally sized images, in declaration order, encoded for their kind. Each layer is also a texture, `<path>#layer<n>`. Repeated loads return the same array."
  doc "Sprites (SpriteDesc.array, og.Sprite.array) and 2D particle emitters draw from it, one draw per array. In a streaming group it loads once the group is resident (refused before, naming the group), draws nothing while the group is away and returns under the same handle."
  param world World "The world whose renderer owns the array."
  param path String "The array's canonical asset id, such as `base/tiles/ground`."
  returns Result.TextureArrayInfo "Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree."
}
```

<sub>Module `load` line 409</sub>

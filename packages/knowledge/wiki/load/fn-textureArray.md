<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-textureArray"></a>

### `og.load.textureArray`

Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color|normal|linear|pixel` and one `layer "<file>"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array.

SpriteDesc.array (world and screen sprites), og.Sprite.array and 2D particle emitters (an emitter's `array` line) draw from its layers, one draw per array whatever layer each shows, with glow and the additive blend; an `array true` flipbook sheet cooks its frames into one (materials and terrain layers do not sample arrays yet). TextureArrayInfo and the frame's texture bytes report it. One in a streaming group's pack loads once its group is resident (refused before, naming the group); when the group unloads the renderer lets it go, og.render.sprite of it is refused and og.Sprite and particles of it draw nothing until the group is resident again, when it comes back under the same handle.

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
  doc "Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color|normal|linear|pixel` and one `layer \"<file>\"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array."
  doc "SpriteDesc.array (world and screen sprites), og.Sprite.array and 2D particle emitters (an emitter's `array` line) draw from its layers, one draw per array whatever layer each shows, with glow and the additive blend; an `array true` flipbook sheet cooks its frames into one (materials and terrain layers do not sample arrays yet). TextureArrayInfo and the frame's texture bytes report it. One in a streaming group's pack loads once its group is resident (refused before, naming the group); when the group unloads the renderer lets it go, og.render.sprite of it is refused and og.Sprite and particles of it draw nothing until the group is resident again, when it comes back under the same handle."
  param world World "The world whose renderer owns the array."
  param path String "The array's asset id, such as `tiles/ground.ogt`."
  returns Result.TextureArrayInfo "Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree."
}
```

<sub>Module `load` line 309</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-texture"></a>

### `og.load.texture`

Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load.

Sources are PNG, JPEG or KTX2 files under assets/ (`texture { kind color|normal|linear|pixel }` in `<file>.ogt`). A KTX2's `KTXorientation` and `KTXswizzle` are honoured: one that is not `rd` and `rgba` is decoded, flipped to rows top to bottom and swizzled, and cooked again; one that is (Basis ETC1S or UASTC, or RGBA8, of a colour or linear kind) is taken as it is.

```ts
og.load.texture(world: World, path: string): Result<Texture>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the texture. |
| `path` | `string` | no | The texture's asset id, such as `textures/stone.png`. |

**Returns** `Result<Texture>`.

Binding symbol `og_load_texture` · publish builds: kept

```ogt
fn texture {
  doc "Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load."
  doc "Sources are PNG, JPEG or KTX2 files under assets/ (`texture { kind color|normal|linear|pixel }` in `<file>.ogt`). A KTX2's `KTXorientation` and `KTXswizzle` are honoured: one that is not `rd` and `rgba` is decoded, flipped to rows top to bottom and swizzled, and cooked again; one that is (Basis ETC1S or UASTC, or RGBA8, of a colour or linear kind) is taken as it is."
  param world World "The world whose renderer owns the texture."
  param path String "The texture's asset id, such as `textures/stone.png`."
  returns Result.Texture
}
```

<sub>`api.ogt` line 8185</sub>

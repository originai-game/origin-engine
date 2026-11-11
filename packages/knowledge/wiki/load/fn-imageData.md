<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-imageData"></a>

### `og.load.imageData`

Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are checked against the budget before it is decoded, off the engine's thread. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why).

Completion is host input, applied at a tick boundary and journaled, so a replay puts it in place at the same tick. It is local to each session: an asset's status is not world state and is in no snapshot or hash.

See `loading §10`.

```ts
og.load.imageData(world: World, name: string, bytes: Uint8Array, desc?: ImageDataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `name` | `string` | no | A name for it in errors and dumps, such as `drawings/cat.png`. |
| `bytes` | `Uint8Array` | no | The file's bytes. |
| `desc` | `ImageDataDesc` | yes | How to read it and its budget. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: over the budget's bytes or textureSize, or not a PNG or JPEG.

Binding symbol `og_load_image_data` · publish builds: kept

```ogt
fn imageData {
  doc "Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are checked against the budget before it is decoded, off the engine's thread. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why)."
  guide "loading §10"
  doc "Completion is host input, applied at a tick boundary and journaled, so a replay puts it in place at the same tick. It is local to each session: an asset's status is not world state and is in no snapshot or hash."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `drawings/cat.png`."
  param bytes Bytes "The file's bytes."
  param desc ImageDataDesc "How to read it and its budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes or textureSize, or not a PNG or JPEG."
}
```

<sub>Module `load` line 551</sub>

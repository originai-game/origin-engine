<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-imageData"></a>

### `og.load.imageData`

Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are read from its header and checked against the budget before it is decoded. On the web the browser decodes it off the engine's thread (createImageBitmap in the engine's Worker), and no decoder is in the wasm; natively a worker thread decodes it. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why).

Completion is host input, applied at a tick boundary and journaled like a key press: natively a load is in place before the tick after the call; on the web it lands at the first boundary after the browser has decoded it, and og.load.completions records that tick, so a replay (og replay --compare, og.load.replayCompletions) puts it in place at the same tick. In a lockstep or rollback room content made by one player is not on the others' machines, and the completion is local to each session (an asset's status is not world state and is in no snapshot or hash): send the bytes with og.platform.roomSend (at most 64 KiB a message), let each peer load them, and let the shared simulation use the content only from a tick the peers agree on through their inputs (og.net.submit), never from og.load.status.

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
  doc "Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are read from its header and checked against the budget before it is decoded. On the web the browser decodes it off the engine's thread (createImageBitmap in the engine's Worker), and no decoder is in the wasm; natively a worker thread decodes it. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why)."
  doc "Completion is host input, applied at a tick boundary and journaled like a key press: natively a load is in place before the tick after the call; on the web it lands at the first boundary after the browser has decoded it, and og.load.completions records that tick, so a replay (og replay --compare, og.load.replayCompletions) puts it in place at the same tick. In a lockstep or rollback room content made by one player is not on the others' machines, and the completion is local to each session (an asset's status is not world state and is in no snapshot or hash): send the bytes with og.platform.roomSend (at most 64 KiB a message), let each peer load them, and let the shared simulation use the content only from a tick the peers agree on through their inputs (og.net.submit), never from og.load.status."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `drawings/cat.png`."
  param bytes Bytes "The file's bytes."
  param desc ImageDataDesc "How to read it and its budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes or textureSize, or not a PNG or JPEG."
}
```

<sub>Module `load` line 523</sub>

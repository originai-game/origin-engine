<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environmentData"></a>

### `og.load.environmentData`

Load an HDR environment from bytes, uncooked, at run time, for Environment.map: an equirectangular Radiance .hdr (or .exr where RenderCapabilities.exrDecode is true), twice as wide as tall. Its size is checked against the budget before it is decoded off the engine's thread; the renderer GGX-prefilters it on the GPU when it first draws with it. The handle is pending until the map is in place, then ready (og.load.environmentInfo) or failed (og.load.error); until then Environment lights by the sky.

Completion is host input, journaled as og.load.imageData's is.

See `loading §10`.

```ts
og.load.environmentData(world: World, name: string, bytes: Uint8Array, desc?: EnvironmentDataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `name` | `string` | no | A name for it in errors and dumps, such as `uploads/sunset.hdr`. |
| `bytes` | `Uint8Array` | no | The file's bytes. |
| `desc` | `EnvironmentDataDesc` | yes | Its face size, intensity and budget. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: over the budget's bytes or textureSize, not twice as wide as tall, a size that is not a power of two from 16 to 2048, not a .hdr or .exr file, or an .exr where RenderCapabilities.exrDecode is false (the reason names it).

Binding symbol `og_load_environment_data` · publish builds: kept

```ogt
fn environmentData {
  doc "Load an HDR environment from bytes, uncooked, at run time, for Environment.map: an equirectangular Radiance .hdr (or .exr where RenderCapabilities.exrDecode is true), twice as wide as tall. Its size is checked against the budget before it is decoded off the engine's thread; the renderer GGX-prefilters it on the GPU when it first draws with it. The handle is pending until the map is in place, then ready (og.load.environmentInfo) or failed (og.load.error); until then Environment lights by the sky."
  guide "loading §10"
  doc "Completion is host input, journaled as og.load.imageData's is."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `uploads/sunset.hdr`."
  param bytes Bytes "The file's bytes."
  param desc EnvironmentDataDesc "Its face size, intensity and budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes or textureSize, not twice as wide as tall, a size that is not a power of two from 16 to 2048, not a .hdr or .exr file, or an .exr where RenderCapabilities.exrDecode is false (the reason names it)."
}
```

<sub>Module `load` line 589</sub>

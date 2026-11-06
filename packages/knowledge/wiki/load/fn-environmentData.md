<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environmentData"></a>

### `og.load.environmentData`

Load an HDR environment from bytes, uncooked, at run time: an equirectangular Radiance .hdr (or OpenEXR .exr where RenderCapabilities.exrDecode is true) that a player uploaded or a generator wrote, for Environment.map as og.load.environment's cooked maps are. The game fetches the file itself (from a URL, an upload or a generator) and passes its bytes. Its size, width and height are read from its header and checked against the budget before it is decoded; the image is twice as wide as it is tall (the centre column faces +X, as three.js maps it). Natively a worker thread decodes it; on the web the .hdr decodes in rows at the tick boundaries, within LoadBudget.frameMs, never in one long stall. The diffuse irradiance (nine harmonics) is gathered as it decodes, and when the renderer first draws with it the image is resampled into a cube and GGX-prefiltered into the five levels a cooked map has (roughness 0 to 1), on the GPU by the passes the sky's own light runs (the cook's `bake` does the same on the CPU for cooked maps and for tools without a GPU). The handle is pending until the map is in place, then ready (og.load.environmentInfo gives the map) or failed (og.load.error says why); until then Environment lights the scene by the sky.

Completion is host input, applied at a tick boundary and journaled as og.load.imageData's is, so a replay puts it in place at the same tick; in a lockstep or rollback room it is local to each session (share the bytes with og.platform.roomSend).

```ts
og.load.environmentData(world: World, name: string, bytes: Uint8Array, desc?: EnvironmentDataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `name` | `string` | no | A name for it in errors and dumps, such as `uploads/sunset.hdr`. |
| `bytes` | `Uint8Array` | no | The file's bytes. |
| `desc` | `EnvironmentDataDesc` | yes | Its face size, intensity and budget. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: over the budget's bytes or textureSize, not twice as wide as tall, a size that is not a power of two from 16 to 2048, not a Radiance .hdr or OpenEXR file, or an .exr where RenderCapabilities.exrDecode is false (the reason names it: convert the image to .hdr, or cook it).

Binding symbol `og_load_environment_data` · publish builds: kept

```ogt
fn environmentData {
  doc "Load an HDR environment from bytes, uncooked, at run time: an equirectangular Radiance .hdr (or OpenEXR .exr where RenderCapabilities.exrDecode is true) that a player uploaded or a generator wrote, for Environment.map as og.load.environment's cooked maps are. The game fetches the file itself (from a URL, an upload or a generator) and passes its bytes. Its size, width and height are read from its header and checked against the budget before it is decoded; the image is twice as wide as it is tall (the centre column faces +X, as three.js maps it). Natively a worker thread decodes it; on the web the .hdr decodes in rows at the tick boundaries, within LoadBudget.frameMs, never in one long stall. The diffuse irradiance (nine harmonics) is gathered as it decodes, and when the renderer first draws with it the image is resampled into a cube and GGX-prefiltered into the five levels a cooked map has (roughness 0 to 1), on the GPU by the passes the sky's own light runs (the cook's `bake` does the same on the CPU for cooked maps and for tools without a GPU). The handle is pending until the map is in place, then ready (og.load.environmentInfo gives the map) or failed (og.load.error says why); until then Environment lights the scene by the sky."
  doc "Completion is host input, applied at a tick boundary and journaled as og.load.imageData's is, so a replay puts it in place at the same tick; in a lockstep or rollback room it is local to each session (share the bytes with og.platform.roomSend)."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `uploads/sunset.hdr`."
  param bytes Bytes "The file's bytes."
  param desc EnvironmentDataDesc "Its face size, intensity and budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes or textureSize, not twice as wide as tall, a size that is not a power of two from 16 to 2048, not a Radiance .hdr or OpenEXR file, or an .exr where RenderCapabilities.exrDecode is false (the reason names it: convert the image to .hdr, or cook it)."
}
```

<sub>`api.ogt` line 8448</sub>

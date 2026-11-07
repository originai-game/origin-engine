<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-vfx"></a>

### `og.load.vfx`

Load a composed effect: an `.ogt` of `type vfx` (the og.vfx module's file: a duration, events and a timeline of emitters, effect meshes, trails, beams, lights, decals, flashes, pixel-effect flipbooks, shakes and sounds), with every emitter, effect material, model, flipbook sheet and sound it names. An Effekseer effect in assets/ (`.efkproj`, `.efkefc`) is one too: the cook converts its nodes into the engine's own vfx, emitters, effect materials and meshes and warns of what it could not carry; `og vfx import` writes the same conversion as `.ogt` text to edit. Loaded once per path; play it with og.vfx.spawn or an og.Vfx whose asset is the handle.

Also known as: Effekseer, efkproj, efkefc, Effekseer import, import a particle effect

```ts
og.load.vfx(world: World, path: string): Result<VfxAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The effect's asset id, such as `fx/fireball-impact.ogt` or `fx/slash.efkproj`. |

**Returns** `Result<VfxAsset>`: Refused for a path not in the bundle (run og cook), a file that is not a vfx, or an asset it names that will not load.

Binding symbol `og_load_vfx` · publish builds: kept

```ogt
fn vfx {
  doc "Load a composed effect: an `.ogt` of `type vfx` (the og.vfx module's file: a duration, events and a timeline of emitters, effect meshes, trails, beams, lights, decals, flashes, pixel-effect flipbooks, shakes and sounds), with every emitter, effect material, model, flipbook sheet and sound it names. An Effekseer effect in assets/ (`.efkproj`, `.efkefc`) is one too: the cook converts its nodes into the engine's own vfx, emitters, effect materials and meshes and warns of what it could not carry; `og vfx import` writes the same conversion as `.ogt` text to edit. Loaded once per path; play it with og.vfx.spawn or an og.Vfx whose asset is the handle."
  aliases "Effekseer, efkproj, efkefc, Effekseer import, import a particle effect"
  param world World "The world."
  param path String "The effect's asset id, such as `fx/fireball-impact.ogt` or `fx/slash.efkproj`."
  returns Result.VfxAsset "Refused for a path not in the bundle (run og cook), a file that is not a vfx, or an asset it names that will not load."
}
```

<sub>Module `load` line 430</sub>

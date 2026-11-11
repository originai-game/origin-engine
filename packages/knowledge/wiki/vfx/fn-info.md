<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="fn-info"></a>

### `og.vfx.info`

What a loaded composed effect holds.

```ts
og.vfx.info(world: World, asset: VfxAsset): Result<VfxInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `asset` | `VfxAsset` | no | The effect. |

**Returns** `Result<VfxInfo>`: Refused for an effect not loaded in this world.

Binding symbol `og_vfx_info` · publish builds: kept

```ogt
fn info {
  doc "What a loaded composed effect holds."
  param world World "The world."
  param asset VfxAsset "The effect."
  returns Result.VfxInfo "Refused for an effect not loaded in this world."
}
```

<sub>Module `vfx` line 89</sub>

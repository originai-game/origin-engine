<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-lut"></a>

### `og.load.lut`

A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle.

```ts
og.load.lut(world: World, path: string): Result<Lut>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `luts/warm.cube`. |

**Returns** `Result<Lut>`: Refused for a path not in the bundle (the reason says to run og cook).

Binding symbol `og_load_lut` · publish builds: kept

```ogt
fn lut {
  doc "A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `luts/warm.cube`."
  returns Result.Lut "Refused for a path not in the bundle (the reason says to run og cook)."
}
```

<sub>`api.ogt` line 8162</sub>

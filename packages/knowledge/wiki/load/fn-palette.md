<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-palette"></a>

### `og.load.palette`

A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle.

```ts
og.load.palette(world: World, path: string): Result<Palette>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `palettes/dusk.hex`. |

**Returns** `Result<Palette>`: Refused for a path not in the bundle (the reason says to run og cook).

Binding symbol `og_load_palette` · publish builds: kept

```ogt
fn palette {
  doc "A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `palettes/dusk.hex`."
  returns Result.Palette "Refused for a path not in the bundle (the reason says to run og cook)."
}
```

<sub>Module `load` line 345</sub>

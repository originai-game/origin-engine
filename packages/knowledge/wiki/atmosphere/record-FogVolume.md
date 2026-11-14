<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-FogVolume"></a>

### `og.FogVolume` (component)

Fog inside the og.Volume on its entity (none: everywhere), lit by the sun through its shadows and by the point, spot and rect lights: mist in a hollow, smoke in a room, the haze a light shaft shows in. It needs no og.Fog and adds to one. Drawn for a perspective camera where the device draws volumetric fog; the 16 nearest the camera.

See `render §7`.

Also known as: local fog, fog in an area, ground mist in a hollow, smoke filling a room, fog bank, haze in a cave, steam, volumetric fog zone

Token `og.FogVolume`, registered under the schema name `og.FogVolume`; its value type is `FogVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The fog's colour: what it tints the light it scatters. |
| `density` | `number` | `0.1` | 0 … 4 | — | tunable | How much light a metre of it takes (its extinction), inside the volume, fading with the volume's `fade` past it. 0.05 is a haze, 0.3 thick smoke. |
| `scattering` | `number` | `0.8` | 0 … 4 | — | tunable | How much of the light it takes it sends on: 0 is black smoke that only dims, 1 a white mist, above 1 brightens its shafts for effect. |

```ogt
component FogVolume {
  aliases "local fog, fog in an area, ground mist in a hollow, smoke filling a room, fog bank, haze in a cave, steam, volumetric fog zone"
  doc "Fog inside the og.Volume on its entity (none: everywhere), lit by the sun through its shadows and by the point, spot and rect lights: mist in a hollow, smoke in a room, the haze a light shaft shows in. It needs no og.Fog and adds to one. Drawn for a perspective camera where the device draws volumetric fog; the 16 nearest the camera."
  guide "render §7"
  field color Color "The fog's colour: what it tints the light it scatters." {
    default 1 1 1
  }
  field density f32 "How much light a metre of it takes (its extinction), inside the volume, fading with the volume's `fade` past it. 0.05 is a haze, 0.3 thick smoke." {
    default 0.1
    min 0
    max 4
    tunable true
  }
  field scattering f32 "How much of the light it takes it sends on: 0 is black smoke that only dims, 1 a white mist, above 1 brightens its shafts for effect." {
    default 0.8
    min 0
    max 4
    tunable true
  }
}
```

<sub>Module `atmosphere` line 96</sub>

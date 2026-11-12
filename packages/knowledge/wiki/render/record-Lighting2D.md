<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Lighting2D"></a>

### `og.Lighting2D` (resource)

2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. A texture with a normal map (`normal auto` or `normal "file.png"` in its import settings' texture block) is shaded by where its surface faces each light, which stands Light2D.height above the plane; a flat area keeps exactly the light it had. Other layers (and the overlay) are drawn as authored. Off by default.

Token `og.Lighting2D`, registered under the schema name `og.Lighting2D`; its value type is `Lighting2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Light the layers. |
| `ambient` | `Color` | `{ r: 0.15, g: 0.15, b: 0.15 }` | — | — | — | Light everywhere (0 is darkness outside the lights). |
| `minLayer` | `number` | `-1000000` | — | — | — | The lowest lit sprite layer. |
| `maxLayer` | `number` | `99` | — | — | — | The highest lit sprite layer (above it: UI and HUD layers stay unlit). |
| `resolution` | `number` | `0.5` | 0.25 … 1 | — | — | The light map's resolution as a fraction of the target. |
| `shadowAngles` | `number` | `1024` | 64 … 4096 | — | — | The angular resolution of each shadow-casting light's shadow: how many directions around it are traced against the occluders (more keeps a small occluder far from a large light sharp, at more cost). |

```ogt
resource Lighting2D {
  doc "2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. A texture with a normal map (`normal auto` or `normal \"file.png\"` in its import settings' texture block) is shaded by where its surface faces each light, which stands Light2D.height above the plane; a flat area keeps exactly the light it had. Other layers (and the overlay) are drawn as authored. Off by default."
  field enabled bool "Light the layers."
  field ambient Color "Light everywhere (0 is darkness outside the lights)." {
    default 0.15 0.15 0.15
  }
  field minLayer i32 "The lowest lit sprite layer." {
    default -1000000
  }
  field maxLayer i32 "The highest lit sprite layer (above it: UI and HUD layers stay unlit)." {
    default 99
  }
  field resolution f32 "The light map's resolution as a fraction of the target." {
    default 0.5
    min 0.25
    max 1
  }
  field shadowAngles u32 "The angular resolution of each shadow-casting light's shadow: how many directions around it are traced against the occluders (more keeps a small occluder far from a large light sharp, at more cost)." {
    default 1024
    min 64
    max 4096
  }
}
```

<sub>Module `render` line 2008</sub>

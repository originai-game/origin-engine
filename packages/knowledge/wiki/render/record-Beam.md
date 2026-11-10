<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Beam"></a>

### `og.Beam` (component)

A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter).

Also known as: laser beam, laser, LineRenderer, tether, lightning bolt

Token `og.Beam`, registered under the schema name `og.Beam`; its value type is `Beam`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | The far end, in world space. |
| `width` | `number` | `0.1` | ≥ 0 | `m` | tunable | The beam's width. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour; above 1 blooms. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `texture` | `Texture` | `0` | — | — | — | A texture along the beam; 0 is solid. |
| `tile` | `number` | `1` | ≥ 0 | — | — | Texture repeats per world unit along the beam. |
| `scroll` | `number` | `0` | — | — | tunable | Texture scroll in repeats per second. |
| `segments` | `number` | `1` | 1 … 64 | — | — | Segments: 1 is straight, more let jitter bend it. |
| `jitter` | `number` | `0` | ≥ 0 | `m` | — | Sideways jitter in world units, new every tick (lightning). |
| `seed` | `number` | `0` | — | — | — | The jitter's seed. |
| `blend` | `StripBlend` | `"additive"` | — | — | — | Additive or alpha blended. |

```ogt
component Beam {
  doc "A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter)."
  aliases "laser beam, laser, LineRenderer, tether, lightning bolt"
  field end Vector3 "The far end, in world space." {
    worldspace world
    unit m
  }
  field width f32 "The beam's width." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field color Color "The linear colour; above 1 blooms." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field texture Texture "A texture along the beam; 0 is solid."
  field tile f32 "Texture repeats per world unit along the beam." {
    default 1
    min 0
  }
  field scroll f32 "Texture scroll in repeats per second." {
    tunable true
  }
  field segments u32 "Segments: 1 is straight, more let jitter bend it." {
    default 1
    min 1
    max 64
  }
  field jitter f32 "Sideways jitter in world units, new every tick (lightning)." {
    min 0
    unit m
  }
  field seed u32 "The jitter's seed."
  field blend StripBlend "Additive or alpha blended."
}
```

<sub>Module `render` line 2077</sub>

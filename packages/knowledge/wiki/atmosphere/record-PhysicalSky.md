<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-PhysicalSky"></a>

### `og.PhysicalSky` (resource)

A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spherical atmosphere of Earth's proportions with multiple scattering (Hillaire 2020). It replaces the gradient of og.Sky and og.DayNight, and the image-based light follows it. With og.DayNight enabled the cycle also takes its light from it: the sun's colour is the light that crosses the air (white at noon, orange then red as it sets, gone below the horizon), the ambient light the scattered sky, and the fog's colour the sky's horizon; `get:og.DayNight` answers those. With og.Sky enabled, og.Sky's sun direction, disc, stars, moon and brightness are drawn through the air. Alone it draws the sky towards the first DirectionalLight. Visual only: drawn from the tick, never written to the world.

Token `og.PhysicalSky`, registered under the schema name `og.PhysicalSky`; its value type is `PhysicalSky`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the sky by scattering. |
| `haze` | `number` | `1` | 0 … 40 | — | tunable | How much haze the air holds, a multiplier on Earth's aerosols (1: a clear day; 0: pure air; 5 to 10: a hazy or smoggy day, a whiter sky and a wider glow round the sun). |
| `airDensity` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on the air's scattering (1: Earth; above 1 a deeper blue and redder sunsets, below 1 a thinner, darker sky). |
| `ozone` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on ozone's absorption, which keeps the sky blue overhead at twilight (0: none, the twilight sky turns grey and yellow). |
| `groundAlbedo` | `Color` | `{ r: 0.3, g: 0.3, b: 0.3 }` | — | — | — | The ground's colour as the air sees it from below: it lights the haze and the sky below the horizon. |
| `altitude` | `number` | `0` | 0 … 100000 | `m` | tunable | The height above sea level of the world's y 0 (the camera's height is added): from high up the sky darkens overhead and the sun stays up longer. |
| `night` | `Color` | `{ r: 0.0022, g: 0.0032, b: 0.0075 }` | — | — | — | The sky's light with the sun far below the horizon (airglow and starlight), so a night is dark blue rather than black and the land under it keeps a deep blue-grey ambient. |

```ogt
resource PhysicalSky {
  doc "A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spherical atmosphere of Earth's proportions with multiple scattering (Hillaire 2020). It replaces the gradient of og.Sky and og.DayNight, and the image-based light follows it. With og.DayNight enabled the cycle also takes its light from it: the sun's colour is the light that crosses the air (white at noon, orange then red as it sets, gone below the horizon), the ambient light the scattered sky, and the fog's colour the sky's horizon; `get:og.DayNight` answers those. With og.Sky enabled, og.Sky's sun direction, disc, stars, moon and brightness are drawn through the air. Alone it draws the sky towards the first DirectionalLight. Visual only: drawn from the tick, never written to the world."
  field enabled bool "Draw the sky by scattering."
  field haze f32 "How much haze the air holds, a multiplier on Earth's aerosols (1: a clear day; 0: pure air; 5 to 10: a hazy or smoggy day, a whiter sky and a wider glow round the sun)." {
    default 1
    min 0
    max 40
    tunable true
  }
  field airDensity f32 "A multiplier on the air's scattering (1: Earth; above 1 a deeper blue and redder sunsets, below 1 a thinner, darker sky)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field ozone f32 "A multiplier on ozone's absorption, which keeps the sky blue overhead at twilight (0: none, the twilight sky turns grey and yellow)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field groundAlbedo Color "The ground's colour as the air sees it from below: it lights the haze and the sky below the horizon." {
    default 0.3 0.3 0.3
  }
  field altitude f32 "The height above sea level of the world's y 0 (the camera's height is added): from high up the sky darkens overhead and the sun stays up longer." {
    min 0
    max 100000
    unit m
    tunable true
  }
  field night Color "The sky's light with the sun far below the horizon (airglow and starlight), so a night is dark blue rather than black and the land under it keeps a deep blue-grey ambient." {
    default 0.0022 0.0032 0.0075
  }
}
```

<sub>Module `atmosphere` line 225</sub>

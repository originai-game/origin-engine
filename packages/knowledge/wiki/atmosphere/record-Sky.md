<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Sky"></a>

### `og.Sky` (resource)

A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it is drawn, clouds and all, captured from the camera a slice a frame, eight frames to a whole capture, so a drifting cloud, a lightning flash or a fast day moves in the reflections and the ambient light within sixteen frames (about a quarter of a second at 60 fps). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself.

Token `og.Sky`, registered under the schema name `og.Sky`; its value type is `Sky`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw and light by this sky. |
| `zenith` | `Color` | `{ r: 0.18, g: 0.36, b: 0.78 }` | — | — | — | Overhead. |
| `horizon` | `Color` | `{ r: 0.78, g: 0.84, b: 0.92 }` | — | — | — | At the horizon. |
| `ground` | `Color` | `{ r: 0.16, g: 0.14, b: 0.12 }` | — | — | — | Below the horizon. |
| `sunDirection` | `Vector3` | `{ x: 0.45, y: 0.8, z: 0.35 }` | — | — | — | Towards the sun (match your DirectionalLight). |
| `sunColor` | `Color` | `{ r: 1, g: 0.95, b: 0.88 }` | — | — | — | The sun's colour. |
| `sunSize` | `number` | `1.8` | 0 … 20 | `deg` | tunable | The sun disc's angular radius (0: no disc). |
| `sunIntensity` | `number` | `40` | ≥ 0 | — | tunable | The disc's brightness. |
| `glow` | `number` | `0.6` | ≥ 0 | — | tunable | The glow around the sun. |
| `glowPower` | `number` | `24` | ≥ 1 | — | tunable | How tight the glow is. |
| `stars` | `number` | `0` | 0 … 1 | `ratio` | tunable | Stars where the sky is dark: 0 none, 1 a full night sky. |
| `brightness` | `number` | `1.1` | ≥ 0 | — | tunable | A multiplier on the whole sky. |
| `moonSize` | `number` | `0` | 0 … 20 | `deg` | tunable | The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it. |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half). |
| `moonIntensity` | `number` | `1.4` | ≥ 0 | — | tunable | The lit moon's brightness. |
| `moonColor` | `Color` | `{ r: 0.9, g: 0.92, b: 1 }` | — | — | — | The moon's colour. |
| `moonDirection` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Towards the moon; zero puts it opposite sunDirection. |

```ogt
resource Sky {
  doc "A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it is drawn, clouds and all, captured from the camera a slice a frame, eight frames to a whole capture, so a drifting cloud, a lightning flash or a fast day moves in the reflections and the ambient light within sixteen frames (about a quarter of a second at 60 fps). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself."
  field enabled bool "Draw and light by this sky."
  field zenith Color "Overhead." {
    default 0.18 0.36 0.78
  }
  field horizon Color "At the horizon." {
    default 0.78 0.84 0.92
  }
  field ground Color "Below the horizon." {
    default 0.16 0.14 0.12
  }
  field sunDirection Vector3 "Towards the sun (match your DirectionalLight)." {
    default 0.45 0.8 0.35
  }
  field sunColor Color "The sun's colour." {
    default 1 0.95 0.88
  }
  field sunSize f32 "The sun disc's angular radius (0: no disc)." {
    default 1.8
    min 0
    max 20
    unit deg
    tunable true
  }
  field sunIntensity f32 "The disc's brightness." {
    default 40
    min 0
    tunable true
  }
  field glow f32 "The glow around the sun." {
    default 0.6
    min 0
    tunable true
  }
  field glowPower f32 "How tight the glow is." {
    default 24
    min 1
    tunable true
  }
  field stars f32 "Stars where the sky is dark: 0 none, 1 a full night sky." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field brightness f32 "A multiplier on the whole sky." {
    default 1.1
    min 0
    tunable true
  }
  field moonSize f32 "The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it." {
    min 0
    max 20
    unit deg
    tunable true
  }
  field moonPhase f32 "How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half)." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field moonIntensity f32 "The lit moon's brightness." {
    default 1.4
    min 0
    tunable true
  }
  field moonColor Color "The moon's colour." {
    default 0.9 0.92 1
  }
  field moonDirection Vector3 "Towards the moon; zero puts it opposite sunDirection."
}
```

<sub>`api.ogt` line 10674</sub>

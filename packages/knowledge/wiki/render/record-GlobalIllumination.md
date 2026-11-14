<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-GlobalIllumination"></a>

### `og.GlobalIllumination` (resource)

Global illumination: light bounces between surfaces, so shade is lit by what stands around it, traced in screen space, with probes, or both (method). The bounce adds to the diffuse ambient light, and the rays that hit something take their share of the ambient light away, as ambient occlusion does (with PostEffects.ao on, its term does that).

Off by default. Read after each tick; it never feeds back into the simulation, and a world that never names this resource keeps the type table, saves and hash it had without it.

See `render §4`.

Token `og.GlobalIllumination`, registered under the schema name `og.GlobalIllumination`; its value type is `GlobalIllumination`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Bounce the light. |
| `method` | `GiMethod` | `"screen"` | — | — | — | In screen space, with probes, or both. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | Scales the bounced light; 1 is physical. With probes the sky's light and emission are not scaled, only what surfaces bounce. |
| `maxDistance` | `number` | `16` | ≥ 0 | `m` | tunable | How far each ray travels at most: the reach of a bounce. A ray also stops where it leaves the screen, its steps spread over what it crosses on screen. With probes (`both`) the rays reach one probe spacing at most: farther, the probes give the light. |
| `thickness` | `number` | `0.5` | ≥ 0 | `m` | tunable | How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. |
| `steps` | `number` | `12` | 4 … 64 | — | — | Steps along each ray. |
| `rays` | `number` | `4` | 1 … 16 | — | — | Rays per pixel each frame: more is smoother and costlier (TAA adds the frames' rays together). |
| `occlusion` | `number` | `1` | 0 … 1 | — | tunable | How much of the ambient light the rays that hit something take away (1 physical, 0 the bounce is only added); not read while PostEffects.ao is on, whose term occludes. |
| `resolution` | `AoResolution` | `"full"` | — | — | — | Full resolution, or half for a quarter of the rays and a depth-aware upsample. With PostEffects.ao on, the AO is drawn at this resolution too. RenderSettings.aoResolution replaces it as it replaces the AO's. |
| `probeSpacing` | `number` | `1` | ≥ 0.05 | `m` | — | With probes: the distance between neighbouring probes of the finest cascade (16 × 8 × 16 probes around the camera); each next cascade's probes are twice as far apart, reaching twice as far. About a third of a room's size indoors, a few metres outdoors. |
| `probeCascades` | `number` | `3` | 1 … 4 | — | — | With probes: how many cascades, each doubling the reach of the one before (the first is updated every frame, the next every second frame, then every fourth). |
| `probeRays` | `number` | `64` | 32 … 256 | — | — | With probes: rays each probe casts when it is updated; more is smoother and costlier. |
| `allowed` | `boolean` | `true` | — | — | — | The graphics quality's switch: false draws none and enabled keeps the game's choice. og.render.applyQuality writes it (false in the low preset, true in medium and high) on a world that has this resource; a game that names the resource after applying a preset applies the preset again. |

```ogt
resource GlobalIllumination {
  doc "Global illumination: light bounces between surfaces, so shade is lit by what stands around it, traced in screen space, with probes, or both (method). The bounce adds to the diffuse ambient light, and the rays that hit something take their share of the ambient light away, as ambient occlusion does (with PostEffects.ao on, its term does that)."
  doc "Off by default. Read after each tick; it never feeds back into the simulation, and a world that never names this resource keeps the type table, saves and hash it had without it."
  guide "render §4"
  field enabled bool "Bounce the light."
  field method GiMethod "In screen space, with probes, or both."
  field intensity f32 "Scales the bounced light; 1 is physical. With probes the sky's light and emission are not scaled, only what surfaces bounce." {
    default 1
    min 0
    tunable true
  }
  field maxDistance f32 "How far each ray travels at most: the reach of a bounce. A ray also stops where it leaves the screen, its steps spread over what it crosses on screen. With probes (`both`) the rays reach one probe spacing at most: farther, the probes give the light." {
    default 16
    min 0
    unit m
    tunable true
  }
  field thickness f32 "How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field steps u32 "Steps along each ray." {
    default 12
    min 4
    max 64
  }
  field rays u32 "Rays per pixel each frame: more is smoother and costlier (TAA adds the frames' rays together)." {
    default 4
    min 1
    max 16
  }
  field occlusion f32 "How much of the ambient light the rays that hit something take away (1 physical, 0 the bounce is only added); not read while PostEffects.ao is on, whose term occludes." {
    default 1
    min 0
    max 1
    tunable true
  }
  field resolution AoResolution "Full resolution, or half for a quarter of the rays and a depth-aware upsample. With PostEffects.ao on, the AO is drawn at this resolution too. RenderSettings.aoResolution replaces it as it replaces the AO's."
  field probeSpacing f32 "With probes: the distance between neighbouring probes of the finest cascade (16 × 8 × 16 probes around the camera); each next cascade's probes are twice as far apart, reaching twice as far. About a third of a room's size indoors, a few metres outdoors." {
    default 1
    min 0.05
    unit m
  }
  field probeCascades u32 "With probes: how many cascades, each doubling the reach of the one before (the first is updated every frame, the next every second frame, then every fourth)." {
    default 3
    min 1
    max 4
  }
  field probeRays u32 "With probes: rays each probe casts when it is updated; more is smoother and costlier." {
    default 64
    min 32
    max 256
  }
  field allowed bool "The graphics quality's switch: false draws none and enabled keeps the game's choice. og.render.applyQuality writes it (false in the low preset, true in medium and high) on a world that has this resource; a game that names the resource after applying a preset applies the preset again." {
    default true
  }
}
```

<sub>Module `render` line 1464</sub>

<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-LensFlare"></a>

### `og.LensFlare` (resource)

Lens flare, as Filament's screen-space flare (after John Chapman): ghosts and a halo mirror the bright parts of the bloom chain's first level through the centre of the frame with chromatic aberration, are blurred, and are added before the tone curve, the halo under a starburst. The sun and up to seven point and spot lights (the ones with the most light at the camera) also cast ghosts of their own, smooth aperture-shaped discs (the sun's 8 % of the frame's height in radius, a light's three times its size on screen up to that; magnified up to 1.5 times along the line through the centre, and fainter as they are), as bright as 0.56 % (the sun) or 0.28 % (a light) of the source's exposed radiance spread over the sun's disc, in proportion to its light as its bloom is (so a dim lamp's are faint, and a small lamp's small), each weighted by how much of it the depth shows, so a sun behind a roof or a lamp behind a pillar fades out; a ghost that would lie on its own source's glare is left out. Runs after bloom, after TAA, at a quarter of the frame, in exposed light, so the exposure and an HDR display apply to it as they do to bloom. With RenderSettings.bloom on, the flare's thresholds apply above bloomThreshold; with bloom off, the bloom chain's first level runs for the flare alone with no threshold of its own, and only the flare is added. In 3D it turns on the depth-normal prepass. Clouds dim the sun's flare through the light they leave the sun, and fog through what it leaves of the sky towards the sun; a single cloud crossing the sun's disc does not.

Off by default. Read after each tick; it never feeds back into the simulation, and a world that never names this resource keeps the type table, saves and hash it had without it.

Token `og.LensFlare`, registered under the schema name `og.LensFlare`; its value type is `LensFlare`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the lens flare. |
| `intensity` | `number` | `0.1` | ≥ 0 | — | tunable | How much of the flare is added (Filament adds its flare at its bloom strength, 0.1). |
| `ghosts` | `number` | `4` | 0 … 16 | — | — | Ghosts per bright spot, along the line through the centre of the frame; 0 is none. |
| `ghostSpacing` | `number` | `0.6` | 0 … 1 | — | tunable | How far apart the ghosts are, as a fraction of the way to the centre. |
| `ghostThreshold` | `number` | `10` | ≥ 0 | — | tunable | The exposed brightness above which a spot casts ghosts (each screen-space tap clamps at 1, display white; a source's own ghosts stay in proportion to its light). |
| `haloRadius` | `number` | `0.28` | 0 … 0.5 | — | tunable | The halo ring's radius, as a fraction of the frame's height. |
| `haloThickness` | `number` | `0.1` | ≥ 0 | — | tunable | The halo ring's thickness, as a fraction of the frame's height; 0 is no halo. |
| `haloThreshold` | `number` | `10` | ≥ 0 | — | tunable | The exposed brightness above which a spot feeds the halo. |
| `chromaticAberration` | `number` | `0.001` | ≥ 0 | — | tunable | How far the screen-space ghosts' red and blue split along the line to the centre, as a fraction of the frame (Filament's default is 0.005). Each ghost also takes one coating tint, amber, teal and violet in turn. |
| `starburst` | `boolean` | `true` | — | — | — | Filament's starburst: the halo streaked by an angular pattern round the centre of the frame that turns as the camera turns (Filament streaks the whole flare; here the ghosts stay smooth discs). |
| `sun` | `number` | `1` | ≥ 0 | — | tunable | The sun's own flare: a disc of the sun's size in the sky (0.27 degrees without a drawn disc), weighted by the share of it the depth shows as sky; the sky round it is kept out of the screen-space ghosts, so the sun flares once. 0 is none (no ghosts of the sun at all); 1 lets its ghosts reach full strength. |
| `lights` | `number` | `1` | ≥ 0 | — | tunable | The point and spot lights' own flares: each a 0.2 m disc of the light's radiance at its position (a spot only seen from inside its cone), hidden where the depth lies more than 0.2 m in front of it; its own bulb is kept out of the screen-space ghosts, so it flares once. 0 is none (no ghosts of those lights at all). |

```ogt
resource LensFlare {
  doc "Lens flare, as Filament's screen-space flare (after John Chapman): ghosts and a halo mirror the bright parts of the bloom chain's first level through the centre of the frame with chromatic aberration, are blurred, and are added before the tone curve, the halo under a starburst. The sun and up to seven point and spot lights (the ones with the most light at the camera) also cast ghosts of their own, smooth aperture-shaped discs (the sun's 8 % of the frame's height in radius, a light's three times its size on screen up to that; magnified up to 1.5 times along the line through the centre, and fainter as they are), as bright as 0.56 % (the sun) or 0.28 % (a light) of the source's exposed radiance spread over the sun's disc, in proportion to its light as its bloom is (so a dim lamp's are faint, and a small lamp's small), each weighted by how much of it the depth shows, so a sun behind a roof or a lamp behind a pillar fades out; a ghost that would lie on its own source's glare is left out. Runs after bloom, after TAA, at a quarter of the frame, in exposed light, so the exposure and an HDR display apply to it as they do to bloom. With RenderSettings.bloom on, the flare's thresholds apply above bloomThreshold; with bloom off, the bloom chain's first level runs for the flare alone with no threshold of its own, and only the flare is added. In 3D it turns on the depth-normal prepass. Clouds dim the sun's flare through the light they leave the sun, and fog through what it leaves of the sky towards the sun; a single cloud crossing the sun's disc does not."
  doc "Off by default. Read after each tick; it never feeds back into the simulation, and a world that never names this resource keeps the type table, saves and hash it had without it."
  field enabled bool "Draw the lens flare."
  field intensity f32 "How much of the flare is added (Filament adds its flare at its bloom strength, 0.1)." {
    default 0.1
    min 0
    tunable true
  }
  field ghosts u32 "Ghosts per bright spot, along the line through the centre of the frame; 0 is none." {
    default 4
    min 0
    max 16
  }
  field ghostSpacing f32 "How far apart the ghosts are, as a fraction of the way to the centre." {
    default 0.6
    min 0
    max 1
    tunable true
  }
  field ghostThreshold f32 "The exposed brightness above which a spot casts ghosts (each screen-space tap clamps at 1, display white; a source's own ghosts stay in proportion to its light)." {
    default 10
    min 0
    tunable true
  }
  field haloRadius f32 "The halo ring's radius, as a fraction of the frame's height." {
    default 0.28
    min 0
    max 0.5
    tunable true
  }
  field haloThickness f32 "The halo ring's thickness, as a fraction of the frame's height; 0 is no halo." {
    default 0.1
    min 0
    tunable true
  }
  field haloThreshold f32 "The exposed brightness above which a spot feeds the halo." {
    default 10
    min 0
    tunable true
  }
  field chromaticAberration f32 "How far the screen-space ghosts' red and blue split along the line to the centre, as a fraction of the frame (Filament's default is 0.005). Each ghost also takes one coating tint, amber, teal and violet in turn." {
    default 0.001
    min 0
    tunable true
  }
  field starburst bool "Filament's starburst: the halo streaked by an angular pattern round the centre of the frame that turns as the camera turns (Filament streaks the whole flare; here the ghosts stay smooth discs)." {
    default true
  }
  field sun f32 "The sun's own flare: a disc of the sun's size in the sky (0.27 degrees without a drawn disc), weighted by the share of it the depth shows as sky; the sky round it is kept out of the screen-space ghosts, so the sun flares once. 0 is none (no ghosts of the sun at all); 1 lets its ghosts reach full strength." {
    default 1
    min 0
    tunable true
  }
  field lights f32 "The point and spot lights' own flares: each a 0.2 m disc of the light's radiance at its position (a spot only seen from inside its cone), hidden where the depth lies more than 0.2 m in front of it; its own bulb is kept out of the screen-space ghosts, so it flares once. 0 is none (no ghosts of those lights at all)." {
    default 1
    min 0
    tunable true
  }
}
```

<sub>Module `render` line 1231</sub>

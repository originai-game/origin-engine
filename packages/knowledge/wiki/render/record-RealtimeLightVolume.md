<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RealtimeLightVolume"></a>

### `og.RealtimeLightVolume` (component)

Relights the LightVolume on the same entity while the game runs: each probe captures the scene around it (six small views), a few a frame, following moving lights, doors and the sun, gaining a bounce each sweep. Shiny surfaces inside reflect the probes' light, not the sky.

Until every probe is relit, and while GlobalIllumination.allowed is false, the box takes its LightVolume's baked probes, else the sky's. Realtime probes win over baked ones; static models keep their lightmaps. Up to eight volumes are drawn. Presentation only; a world that never names it keeps its type table, saves and hash.

See `render §4`.

Token `og.RealtimeLightVolume`, registered under the schema name `og.RealtimeLightVolume`; its value type is `RealtimeLightVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `updatesPerFrame` | `number` | `2` | ≤ 8 | — | — | Probes relit a frame once every one has been (at most 8 a frame over every volume; the first sweep takes 8). 0 relights them once and keeps them: a bake at run time. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | A multiplier on the light its probes capture; 1 is physical. |

```ogt
component RealtimeLightVolume {
  doc "Relights the LightVolume on the same entity while the game runs: each probe captures the scene around it (six small views), a few a frame, following moving lights, doors and the sun, gaining a bounce each sweep. Shiny surfaces inside reflect the probes' light, not the sky."
  doc "Until every probe is relit, and while GlobalIllumination.allowed is false, the box takes its LightVolume's baked probes, else the sky's. Realtime probes win over baked ones; static models keep their lightmaps. Up to eight volumes are drawn. Presentation only; a world that never names it keeps its type table, saves and hash."
  guide "render §4"
  field updatesPerFrame u32 "Probes relit a frame once every one has been (at most 8 a frame over every volume; the first sweep takes 8). 0 relights them once and keeps them: a bake at run time." {
    default 2
    max 8
  }
  field intensity f32 "A multiplier on the light its probes capture; 1 is physical." {
    default 1
    min 0
    tunable true
  }
}
```

<sub>Module `render` line 4187</sub>

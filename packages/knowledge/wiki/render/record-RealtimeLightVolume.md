<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RealtimeLightVolume"></a>

### `og.RealtimeLightVolume` (component)

Relights the LightVolume on the same entity while the game runs, so its probes follow moving lights, opening doors, a sun crossing the sky and switched lamps: each probe is relit by capturing the scene around it (six small views drawn with every light, the sky's and the lights' shadows, emissive surfaces and lightmapped models), a few probes a frame in an order spread over the box. The captures see the probes too, so light bounces once more each time the box is swept: a lamp lit in a red room fills it with red light. The probes relight reflections too: inside the box a shiny surface reflects the probes' light along the reflected ray (a soft image of the room: a red wall on the side it faces, a lit ceiling above, dark below) instead of the sky. That is too soft for a mirror: inside a ReflectionProbe's box (a realtime one follows the lights) smooth surfaces reflect its capture, drawn with the probes' light, blurring into the probes' own as roughness rises. The probes give no contact shadow where surfaces meet: pair them with PostEffects.ao or GlobalIllumination for darkened corners and grounded objects.

Its probes are sampled once every one has been relit (a few frames after it is added, or after its LightVolume is resized); until then, and while the quality turns global illumination off (og.render.applyQuality's low preset, GlobalIllumination.allowed false), the box takes the baked probes of its LightVolume when the scene has a bake (og bake lightmaps), else the sky's. Where both are drawn the realtime probes win; static models keep their lightmaps. Up to eight light volumes are drawn: the baked ones, then the realtime ones nearest the camera. Keep the box's faces a little inside the walls (a probe on a wall sees into it): its light reaches one spacing beyond its faces, fading out, so the walls, floor and ceiling just outside it take it too (and so would the far side of a wall thinner than a spacing). Read after each tick; it never feeds back into the simulation, and a world that never names it keeps the type table, saves and hash it had without it.

Token `og.RealtimeLightVolume`, registered under the schema name `og.RealtimeLightVolume`; its value type is `RealtimeLightVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `updatesPerFrame` | `number` | `2` | ≤ 8 | — | — | Probes relit a frame once every one has been (at most 8 a frame over every volume; the first sweep takes 8). 0 relights them once and keeps them: a bake at run time. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | A multiplier on the light its probes capture; 1 is physical. |

```ogt
component RealtimeLightVolume {
  doc "Relights the LightVolume on the same entity while the game runs, so its probes follow moving lights, opening doors, a sun crossing the sky and switched lamps: each probe is relit by capturing the scene around it (six small views drawn with every light, the sky's and the lights' shadows, emissive surfaces and lightmapped models), a few probes a frame in an order spread over the box. The captures see the probes too, so light bounces once more each time the box is swept: a lamp lit in a red room fills it with red light. The probes relight reflections too: inside the box a shiny surface reflects the probes' light along the reflected ray (a soft image of the room: a red wall on the side it faces, a lit ceiling above, dark below) instead of the sky. That is too soft for a mirror: inside a ReflectionProbe's box (a realtime one follows the lights) smooth surfaces reflect its capture, drawn with the probes' light, blurring into the probes' own as roughness rises. The probes give no contact shadow where surfaces meet: pair them with PostEffects.ao or GlobalIllumination for darkened corners and grounded objects."
  doc "Its probes are sampled once every one has been relit (a few frames after it is added, or after its LightVolume is resized); until then, and while the quality turns global illumination off (og.render.applyQuality's low preset, GlobalIllumination.allowed false), the box takes the baked probes of its LightVolume when the scene has a bake (og bake lightmaps), else the sky's. Where both are drawn the realtime probes win; static models keep their lightmaps. Up to eight light volumes are drawn: the baked ones, then the realtime ones nearest the camera. Keep the box's faces a little inside the walls (a probe on a wall sees into it): its light reaches one spacing beyond its faces, fading out, so the walls, floor and ceiling just outside it take it too (and so would the far side of a wall thinner than a spacing). Read after each tick; it never feeds back into the simulation, and a world that never names it keeps the type table, saves and hash it had without it."
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

<sub>Module `render` line 3912</sub>

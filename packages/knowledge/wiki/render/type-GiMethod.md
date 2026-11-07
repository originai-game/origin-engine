<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GiMethod"></a>

### `GiMethod` (enum)

How GlobalIllumination bounces the light.

A string: one of the variants below. The zero value is `"screen"`.

| Variant | Doc |
|---|---|
| `"screen"` | In screen space: rays through the depth-normal prepass take the light the previous frame showed where they hit. Fine detail and contact, but nothing from off screen or behind what the screen shows. |
| `"probes"` | With probes: a grid of probes around the camera, each casting rays through a simplified copy of the scene (its meshes and terrains) every few frames, gathers the light arriving from every side, sky included; every lit surface takes its ambient light from the probes around it, tested for which of them see it. Light from off screen and behind walls bounces, a red wall lights the floor beside it however the camera looks, and the sky's light is occluded under roofs, in rooms and between buildings. Coarse (a probe every probeSpacing metres): pair it with PostEffects.ao for contact. |
| `"both"` | Probes for the light from the whole scene and the sky, the screen-space rays for the detail they see. |

```ogt
enum GiMethod {
  doc "How GlobalIllumination bounces the light."
  variant screen "In screen space: rays through the depth-normal prepass take the light the previous frame showed where they hit. Fine detail and contact, but nothing from off screen or behind what the screen shows."
  variant probes "With probes: a grid of probes around the camera, each casting rays through a simplified copy of the scene (its meshes and terrains) every few frames, gathers the light arriving from every side, sky included; every lit surface takes its ambient light from the probes around it, tested for which of them see it. Light from off screen and behind walls bounces, a red wall lights the floor beside it however the camera looks, and the sky's light is occluded under roofs, in rooms and between buildings. Coarse (a probe every probeSpacing metres): pair it with PostEffects.ao for contact."
  variant both "Probes for the light from the whole scene and the sky, the screen-space rays for the detail they see."
}
```

<sub>Module `render` line 1306</sub>

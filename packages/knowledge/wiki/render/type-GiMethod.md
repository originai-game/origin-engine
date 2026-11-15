<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GiMethod"></a>

### `GiMethod` (enum)

How GlobalIllumination bounces the light.

See `render §4`.

A string: one of the variants below. The zero value is `"screen"`.

| Variant | Doc |
|---|---|
| `"screen"` | In screen space: rays through the depth-normal prepass take the light the previous frame showed where they hit. Fine detail and contact, but nothing from off screen or behind what the screen shows. |
| `"probes"` | With probes: a grid of probes around the camera, each casting rays through a simplified copy of the scene (meshes and terrains) every few frames, gathers light from every side, sky included; every lit surface takes its ambient light from the nearby probes that see it. Coarse: probeSpacing apart. |
| `"both"` | Probes for the light from the whole scene and the sky, the screen-space rays for the detail they see. |

```ogt
enum GiMethod {
  doc "How GlobalIllumination bounces the light."
  guide "render §4"
  variant screen "In screen space: rays through the depth-normal prepass take the light the previous frame showed where they hit. Fine detail and contact, but nothing from off screen or behind what the screen shows."
  variant probes "With probes: a grid of probes around the camera, each casting rays through a simplified copy of the scene (meshes and terrains) every few frames, gathers light from every side, sky included; every lit surface takes its ambient light from the nearby probes that see it. Coarse: probeSpacing apart."
  variant both "Probes for the light from the whole scene and the sky, the screen-space rays for the detail they see."
}
```

<sub>Module `render` line 1451</sub>

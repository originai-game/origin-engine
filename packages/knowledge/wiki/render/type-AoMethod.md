<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-AoMethod"></a>

### `AoMethod` (enum)

Which ambient occlusion algorithm runs.

See `render §5`.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | GTAO. |
| `"gtao"` | Ground-truth ambient occlusion (3 slices): the better looking one. |
| `"ssao"` | Hemisphere SSAO with 8 samples: the cheaper one. |
| `"screen"` | Depth-only disc AO in screen units (14 taps, radius clamp(1.6 / depth, 0.004, 0.06) of the screen width, fading out from 120 to 300 m, squared), the post-process SSAO of three.js-era games. It ignores normals; aoRadius and aoRadiusScale do not apply, and aoIntensity scales the occlusion. |

```ogt
enum AoMethod {
  doc "Which ambient occlusion algorithm runs."
  guide "render §5"
  variant auto "GTAO."
  variant gtao "Ground-truth ambient occlusion (3 slices): the better looking one."
  variant ssao "Hemisphere SSAO with 8 samples: the cheaper one."
  variant screen "Depth-only disc AO in screen units (14 taps, radius clamp(1.6 / depth, 0.004, 0.06) of the screen width, fading out from 120 to 300 m, squared), the post-process SSAO of three.js-era games. It ignores normals; aoRadius and aoRadiusScale do not apply, and aoIntensity scales the occlusion."
}
```

<sub>Module `render` line 886</sub>

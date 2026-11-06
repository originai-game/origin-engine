<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ScreenBlur"></a>

### `ScreenBlur` (enum)

Which way PostEffects.screenBlur smears the whole frame.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Off. |
| `"zoom"` | Along the line to screenBlurCenter, each way by screenBlurStrength's fraction of the distance to it (0.1 a light dash, 0.3 a hard impact): a dash forwards, a hit, a warp. |
| `"spin"` | Round screenBlurCenter, each way by screenBlurStrength radians (0.05 is about 3 degrees), on circles in pixels whatever the frame's aspect: a spin, a vortex, a dizzy spell. |
| `"directional"` | Along screenBlurAngle across the whole frame, each way by screenBlurStrength pixels at 720 lines (scaled with the frame's height): a fast pan, a sideways dash. |

```ogt
enum ScreenBlur {
  doc "Which way PostEffects.screenBlur smears the whole frame."
  variant none "Off."
  variant zoom "Along the line to screenBlurCenter, each way by screenBlurStrength's fraction of the distance to it (0.1 a light dash, 0.3 a hard impact): a dash forwards, a hit, a warp."
  variant spin "Round screenBlurCenter, each way by screenBlurStrength radians (0.05 is about 3 degrees), on circles in pixels whatever the frame's aspect: a spin, a vortex, a dizzy spell."
  variant directional "Along screenBlurAngle across the whole frame, each way by screenBlurStrength pixels at 720 lines (scaled with the frame's height): a fast pan, a sideways dash."
}
```

<sub>`api.ogt` line 4624</sub>

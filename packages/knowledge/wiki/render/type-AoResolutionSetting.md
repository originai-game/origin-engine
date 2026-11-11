<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-AoResolutionSetting"></a>

### `AoResolutionSetting` (enum)

The resolution RenderSettings.aoResolution draws ambient occlusion at.

A string: one of the variants below. The zero value is `"postEffects"`.

| Variant | Doc |
|---|---|
| `"postEffects"` | PostEffects.aoResolution's: the game's choice. |
| `"full"` | Full resolution whatever PostEffects.aoResolution says (the high preset). |
| `"half"` | Half resolution whatever PostEffects.aoResolution says (the low preset). |

```ogt
enum AoResolutionSetting {
  doc "The resolution RenderSettings.aoResolution draws ambient occlusion at."
  variant postEffects "PostEffects.aoResolution's: the game's choice."
  variant full "Full resolution whatever PostEffects.aoResolution says (the high preset)."
  variant half "Half resolution whatever PostEffects.aoResolution says (the low preset)."
}
```

<sub>Module `render` line 597</sub>

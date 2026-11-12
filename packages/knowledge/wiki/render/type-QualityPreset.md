<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-QualityPreset"></a>

### `QualityPreset` (enum)

The preset RenderSettings' quality fields were last written with (RenderSettings.quality).

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No preset applied: the fields are their defaults or the game's own. |
| `"low"` | og.render.applyQuality(world, "low"). |
| `"medium"` | og.render.applyQuality(world, "medium"). |
| `"high"` | og.render.applyQuality(world, "high"). |

```ogt
enum QualityPreset {
  doc "The preset RenderSettings' quality fields were last written with (RenderSettings.quality)."
  variant none "No preset applied: the fields are their defaults or the game's own."
  variant low "og.render.applyQuality(world, \"low\")."
  variant medium "og.render.applyQuality(world, \"medium\")."
  variant high "og.render.applyQuality(world, \"high\")."
}
```

<sub>Module `render` line 632</sub>

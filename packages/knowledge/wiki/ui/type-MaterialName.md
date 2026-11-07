<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MaterialName"></a>

### `MaterialName` (enum)

What a layer composites through: a built-in UI material, or a custom one. params is read per material.

A string: one of the variants below. The zero value is `"plain"`.

| Variant | Doc |
|---|---|
| `"plain"` | The layer as drawn. |
| `"dissolve"` | [threshold 0–1, edge width, noise scale (CSS px), _], [edge colour rgba]. |
| `"shine"` | [position 0–1 across the diagonal (negative sweeps by the clock at -position a second), width, angle (rad), intensity], [colour rgba]. |
| `"hologram"` | [scanline spacing (CSS px), flicker 0–1, chromatic split (CSS px), scanline strength], [tint rgba]. |
| `"glitch"` | [strength 0–1, slice height (CSS px), split (CSS px), rate (Hz)]. |
| `"ripple"` | [amplitude (CSS px), wavelength (CSS px), speed (CSS px/s), _], [centre x, y as fractions]. |
| `"pixelate"` | [cell size (CSS px)]. |
| `"colorAdjust"` | [hue (rad), saturation, brightness, grayscale 0–1]. |
| `"outline"` | [width (CSS px), _, _, _], [colour rgba]: an outline of the layer's alpha. |
| `"blur"` | [radius (CSS px)]. |
| `"custom"` | LayerDesc.custom, a `type ui-material`. |

```ogt
enum MaterialName {
  doc "What a layer composites through: a built-in UI material, or a custom one. params is read per material."
  variant plain "The layer as drawn."
  variant dissolve "[threshold 0–1, edge width, noise scale (CSS px), _], [edge colour rgba]."
  variant shine "[position 0–1 across the diagonal (negative sweeps by the clock at -position a second), width, angle (rad), intensity], [colour rgba]."
  variant hologram "[scanline spacing (CSS px), flicker 0–1, chromatic split (CSS px), scanline strength], [tint rgba]."
  variant glitch "[strength 0–1, slice height (CSS px), split (CSS px), rate (Hz)]."
  variant ripple "[amplitude (CSS px), wavelength (CSS px), speed (CSS px/s), _], [centre x, y as fractions]."
  variant pixelate "[cell size (CSS px)]."
  variant colorAdjust "[hue (rad), saturation, brightness, grayscale 0–1]."
  variant outline "[width (CSS px), _, _, _], [colour rgba]: an outline of the layer's alpha."
  variant blur "[radius (CSS px)]."
  variant custom "LayerDesc.custom, a `type ui-material`."
}
```

<sub>Module `ui` line 7265</sub>

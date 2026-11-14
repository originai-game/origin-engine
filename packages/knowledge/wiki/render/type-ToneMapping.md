<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ToneMapping"></a>

### `ToneMapping` (enum)

The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen.

A string: one of the variants below. The zero value is `"aces"`.

| Variant | Doc |
|---|---|
| `"aces"` | ACES (the fitted filmic curve), as three.js ACESFilmicToneMapping: the default. |
| `"agx"` | AgX (Blender 4's and three.js AgXToneMapping's fit, base look): bright saturated lights go to white instead of skewing hue, and keep their hue further up than under ACES. |
| `"none"` | No curve: the exposed linear colour is clamped to 0..1, so 2D colours come out exactly as authored at exposure 1 (the same as toneMapped false). |
| `"neutral"` | Khronos PBR Neutral, as three.js NeutralToneMapping: colours up to about 0.8 come out as authored (a white-lit base colour shows as the colour picked), only brighter ones roll off to white. For product shots and saturated stylised colour that ACES would darken and shift. |

```ogt
enum ToneMapping {
  doc "The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen."
  variant aces "ACES (the fitted filmic curve), as three.js ACESFilmicToneMapping: the default."
  variant agx "AgX (Blender 4's and three.js AgXToneMapping's fit, base look): bright saturated lights go to white instead of skewing hue, and keep their hue further up than under ACES."
  variant none "No curve: the exposed linear colour is clamped to 0..1, so 2D colours come out exactly as authored at exposure 1 (the same as toneMapped false)."
  variant neutral "Khronos PBR Neutral, as three.js NeutralToneMapping: colours up to about 0.8 come out as authored (a white-lit base colour shows as the colour picked), only brighter ones roll off to white. For product shots and saturated stylised colour that ACES would darken and shift."
}
```

<sub>Module `render` line 627</sub>

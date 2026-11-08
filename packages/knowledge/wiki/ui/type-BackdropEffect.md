<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BackdropEffect"></a>

### `BackdropEffect` (desc struct)

What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `blur` | `number` | `16` | ≥ 0 | `cssPx` | — | The blur. |
| `saturation` | `number` | `1` | ≥ 0 | `ratio` | — | Multiplies the saturation. |
| `brightness` | `number` | `1` | ≥ 0 | `ratio` | — | Multiplies the brightness. |
| `tint` | `Paint \| null` | `null` | — | — | — | Over the blur, by its alpha; null is the theme's panel at 60 % (with a glass weight, the weight's tint). |
| `glass` | `GlassWeight` | `"none"` | — | — | — | One of flat glass's weights (docs/ui.md §3.3): it sets the blur, 200 % saturation, the tint (unless tint is given), a 0.5 px hairline inside the edge and CSS mixing, in the theme's scheme; under reduced transparency the solid raised surface, under high contrast solid with a strong 1 px edge. none is the fields as given. |
| `elevation` | `Elevation` | `"none"` | — | — | — | The soft shadow outside it, as the design language's shadows (with a glass weight, raised by default). |
| `edge` | `number` | `0` | ≥ 0 | `cssPx` | — | A hairline just inside the edge (CSS inset 0 0 0 <edge>px); 0 is none, or the weight's. |
| `edgeColor` | `Paint \| null` | `null` | — | — | — | The hairline's colour; null is the weight's (or the theme's line). |
| `solid` | `boolean` | `false` | — | — | — | The tint drawn opaque, with the hairline and shadow: nothing under it is read (reduced transparency, increased contrast). |
| `cssMix` | `boolean` | `false` | — | — | — | Mix the tint, hairline and shadow in sRGB-encoded values, as CSS does, so a tint reads as it does on a web page (glass weights always do). |

```ogt
struct BackdropEffect {
  doc "What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass."
  desc true
  field blur f32 "The blur." {
    default 16
    min 0
    unit cssPx
  }
  field saturation f32 "Multiplies the saturation." {
    default 1
    min 0
    unit ratio
  }
  field brightness f32 "Multiplies the brightness." {
    default 1
    min 0
    unit ratio
  }
  field tint Option.Paint "Over the blur, by its alpha; null is the theme's panel at 60 % (with a glass weight, the weight's tint)."
  field glass GlassWeight "One of flat glass's weights (docs/ui.md §3.3): it sets the blur, 200 % saturation, the tint (unless tint is given), a 0.5 px hairline inside the edge and CSS mixing, in the theme's scheme; under reduced transparency the solid raised surface, under high contrast solid with a strong 1 px edge. none is the fields as given."
  field elevation Elevation "The soft shadow outside it, as the design language's shadows (with a glass weight, raised by default)."
  field edge f32 "A hairline just inside the edge (CSS inset 0 0 0 <edge>px); 0 is none, or the weight's." {
    min 0
    unit cssPx
  }
  field edgeColor Option.Paint "The hairline's colour; null is the weight's (or the theme's line)."
  field solid bool "The tint drawn opaque, with the hairline and shadow: nothing under it is read (reduced transparency, increased contrast)."
  field cssMix bool "Mix the tint, hairline and shadow in sRGB-encoded values, as CSS does, so a tint reads as it does on a web page (glass weights always do)."
}
```

<sub>Module `ui` line 3504</sub>

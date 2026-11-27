<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiDesc"></a>

### `UiDesc` (desc struct)

How the UI looks: the scheme, its fonts, the contrast, the seed colours its palette is derived from, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it.

Also known as: UI scale, text scale, font size, accessibility, high contrast, reduced motion, UI settings

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette. |
| `font` | `Font \| null` | `null` | — | — | — | The body font, from og.load.font; null measures text at half an em per character and draws none. |
| `displayFont` | `Font \| null` | `null` | — | — | — | The display font; null uses the body font. |
| `monoFont` | `Font \| null` | `null` | — | — | — | The monospaced font; null uses the body font. |
| `roundedFont` | `Font \| null` | `null` | — | — | — | The rounded font (levels and scores); null uses the display font, then the body font. |
| `fallbackFonts` | `readonly Font[]` | `[]` | — | — | — | Fonts for other scripts (Hebrew, Devanagari, a CJK face, each fetched the first time its script shows), tried in order after the locale's (og.i18n) for text the role's font cannot draw: each run of a mixed text takes the first font that has it. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast). |
| `seeds` | `UiSeeds \| null` | `null` | — | — | — | The seed colours the whole palette is derived from, over the style's (docs/ui.md §3.1): a faction's or the time of day's, given again at run time to restyle the whole UI. Given whole: a seed left out takes the style's. null keeps the style's seeds. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion). |
| `textScale` | `number` | `1` | 1 … 2 | `ratio` | — | The player's text size (a settings screen's slider): every text size and leading is multiplied by it and the layout grows around the text, never clipping it. 1 is the style's sizes, 1.5 the largest (a larger one lays out at 1.5). |
| `unitScale` | `number` | `0` | 0 … 2 | `ratio` | — | The UI unit in CSS px: the TSX tree lays out in units and draws them at it, so every length grows with the surface; the probe reads boxes in surface px. 0 is auto: on a mouse or pad surface its 1080p-equivalent height over 720, 1 to 2; a touch surface keeps 1. 1 to 2 pins it. |
| `colorVision` | `ColorVision` | `"none"` | — | — | — | The player's colour vision (a colour-blind mode): the whole frame, scene and UI, is recoloured for it after every other effect (colorVisionFilter); none (the default) draws it as it is. Viénot 1999 and Brettel 1997 simulate; Fidaner 2005's daltonisation corrects. |
| `colorVisionFilter` | `ColorVisionFilter` | `"correct"` | — | — | — | What colorVision does to the frame: correct (the default; the player's setting) or simulate (how a viewer with it sees the frame, for checking a screen). |
| `colorVisionStrength` | `number` | `1` | 0 … 1 | `ratio` | — | The deficiency's severity, 0 to 1: 1 (the default) a dichromat's; less blends towards the frame as it is, for the commoner anomalous trichromacy. |
| `icons` | `IconAtlasDesc \| null` | `null` | — | — | — | The icon atlas a document's `<og-icon>` draws from; null takes the style's. |
| `images` | `readonly UiInlineImage[]` | `[]` | — | — | — | Pictures rich text draws inline by name ([img=<name>]): a portrait in a line of dialogue, an item's art in a tooltip. Any texture, not only the icon atlas's cells; each one em tall at its aspect, in its own colours. |
| `style` | `Style \| null` | `null` | — | — | — | A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the default style (fonts, icons and a primary colour given here still win); null is the default style, the design language the engine shares with origin-agent. |
| `sounds` | `UiSounds \| null` | `null` | — | — | — | Sounds for UI events (they win over the style's). |

```ogt
struct UiDesc {
  doc "How the UI looks: the scheme, its fonts, the contrast, the seed colours its palette is derived from, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it."
  aliases "UI scale, text scale, font size, accessibility, high contrast, reduced motion, UI settings"
  desc true
  field scheme Scheme "The palette."
  field font Option.Font "The body font, from og.load.font; null measures text at half an em per character and draws none."
  field displayFont Option.Font "The display font; null uses the body font."
  field monoFont Option.Font "The monospaced font; null uses the body font."
  field roundedFont Option.Font "The rounded font (levels and scores); null uses the display font, then the body font."
  field fallbackFonts List.Font "Fonts for other scripts (Hebrew, Devanagari, a CJK face, each fetched the first time its script shows), tried in order after the locale's (og.i18n) for text the role's font cannot draw: each run of a mixed text takes the first font that has it."
  field contrast Contrast "Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast)."
  field seeds Option.UiSeeds "The seed colours the whole palette is derived from, over the style's (docs/ui.md §3.1): a faction's or the time of day's, given again at run time to restyle the whole UI. Given whole: a seed left out takes the style's. null keeps the style's seeds."
  field reducedMotion bool "Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion)."
  field textScale f32 "The player's text size (a settings screen's slider): every text size and leading is multiplied by it and the layout grows around the text, never clipping it. 1 is the style's sizes, 1.5 the largest (a larger one lays out at 1.5)." {
    default 1
    min 1
    max 2
    unit ratio
  }
  field unitScale f32 "The UI unit in CSS px: the TSX tree lays out in units and draws them at it, so every length grows with the surface; the probe reads boxes in surface px. 0 is auto: on a mouse or pad surface its 1080p-equivalent height over 720, 1 to 2; a touch surface keeps 1. 1 to 2 pins it." {
    default 0
    min 0
    max 2
    unit ratio
  }
  field colorVision ColorVision "The player's colour vision (a colour-blind mode): the whole frame, scene and UI, is recoloured for it after every other effect (colorVisionFilter); none (the default) draws it as it is. Viénot 1999 and Brettel 1997 simulate; Fidaner 2005's daltonisation corrects."
  field colorVisionFilter ColorVisionFilter "What colorVision does to the frame: correct (the default; the player's setting) or simulate (how a viewer with it sees the frame, for checking a screen)."
  field colorVisionStrength f32 "The deficiency's severity, 0 to 1: 1 (the default) a dichromat's; less blends towards the frame as it is, for the commoner anomalous trichromacy." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field icons Option.IconAtlasDesc "The icon atlas a document's `<og-icon>` draws from; null takes the style's."
  field images List.UiInlineImage "Pictures rich text draws inline by name ([img=<name>]): a portrait in a line of dialogue, an item's art in a tooltip. Any texture, not only the icon atlas's cells; each one em tall at its aspect, in its own colours."
  field style Option.Style "A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the default style (fonts, icons and a primary colour given here still win); null is the default style, the design language the engine shares with origin-agent."
  field sounds Option.UiSounds "Sounds for UI events (they win over the style's)."
}
```

<sub>Module `ui` line 80</sub>

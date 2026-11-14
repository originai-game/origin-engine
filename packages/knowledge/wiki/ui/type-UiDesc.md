<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiDesc"></a>

### `UiDesc` (desc struct)

How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it.

Also known as: UI scale, text scale, font size, accessibility, high contrast, reduced motion, UI settings

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette. |
| `density` | `DensityMode` | `"auto"` | — | — | — | The control sizes. |
| `font` | `Font \| null` | `null` | — | — | — | The body font, from og.load.font; null measures text at half an em per character and draws none. |
| `displayFont` | `Font \| null` | `null` | — | — | — | The display font; null uses the body font. |
| `monoFont` | `Font \| null` | `null` | — | — | — | The monospaced font; null uses the body font. |
| `roundedFont` | `Font \| null` | `null` | — | — | — | The rounded font (levels and scores); null uses the display font, then the body font. |
| `fallbackFonts` | `readonly Font[]` | `[]` | — | — | — | Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. A text mixing scripts takes each run from the first font that has it (the role's for its Latin, a fallback for its CJK or Arabic), one label in several fonts. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast). |
| `primaryColor` | `Color \| null` | `null` | — | — | — | An sRGB colour as the game's primary (its hover, pressed and soft tones derived from it, the ink on it whichever of ink and inkInverse reads better); null keeps the style's. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion). |
| `textScale` | `number` | `1` | 1 … 2 | `ratio` | — | The player's text size (a settings screen's slider, kept with the other settings): every text size and leading is multiplied by it and the layout grows around the text (control, row and header heights by the leading body text gains), never clipping it. 1 is the style's sizes, 1.5 the largest (a larger one lays out at 1.5): docs/ui.md §6.1 holds every screen to 1.5, 200 % deferred. |
| `unitScale` | `number` | `0` | 0 … 2 | `ratio` | — | The og-ui unit in CSS px (docs/ui.md §6.1): the UI lays out in units (og.ui.viewport and og.ui.frameViewport are the surface over it) and draws them at it, so every length grows with the surface (the style's text, space, radii and controls, and a game's own px, alike) and the player's textScale multiplies over it. 0 is auto: on a mouse or pad surface its 1080p-equivalent height (the shorter of the height and the width's 16:9 height) over 720, 1 to 2, so a 1280x720 layout fills a 1080p window with body text at 21 CSS px and the smallest step at 18; a touch surface keeps CSS px (1). A value of 1 to 2 pins it (a game that lays out for one size). |
| `reducedTransparency` | `boolean` | `false` | — | — | — | Nothing is see-through: glass parts (a style's glass line) are opaque surfaces (no blur of the frame under them, their frame whole), the scrim is the opaque colour it makes over backdrop and the translucent fills (control, controlStrong, primarySoft, hud) the opaque colours they make over panel. The page's prefers-reduced-transparency turns it on too (Viewport.reducedTransparency); UiTheme.reducedTransparency says it is on. |
| `colorVision` | `ColorVision` | `"none"` | — | — | — | The player's colour vision (a settings screen's colour-blind mode): the whole frame, the scene and the UI alike, is recoloured for it after every other effect (colorVisionFilter), and a document's CSS reads it as @media (-og-color-vision: protanopia) to add the cues it needs. none (the default) draws the frame as it is. Simulation uses Viénot 1999 for protanopia and deuteranopia and Brettel 1997 for tritanopia; correction is Fidaner 2005's daltonisation over them (docs/ui.md §3.2). |
| `colorVisionFilter` | `ColorVisionFilter` | `"correct"` | — | — | — | What colorVision does to the frame: correct (the default; the player's setting) or simulate (how a viewer with it sees the frame, for checking a screen). |
| `colorVisionStrength` | `number` | `1` | 0 … 1 | `ratio` | — | The deficiency's severity, 0 to 1: 1 (the default) a dichromat's; less blends towards the frame as it is, for the commoner anomalous trichromacy. |
| `icons` | `IconAtlasDesc \| null` | `null` | — | — | — | The icon atlas a document's `<og-icon>` draws from; null takes the style's. |
| `fallbackIcons` | `IconAtlasDesc \| null` | `null` | — | — | — | The icon set that draws the names the atlas lacks (icons, or the style's when icons is null): a game's whole set, such as the UI kit's iconoir set, under a style pack's few painted icons, which keep the names they paint. A document's `<og-icon>`, rich text's [icon=], an action's glyph and an art slot's placeholder all look in it; a name in neither warns. null is none. |
| `images` | `readonly UiInlineImage[]` | `[]` | — | — | — | Pictures rich text draws inline by name ([img=<name>]): a portrait in a line of dialogue, an item's art in a tooltip. Any texture, not only the icon atlas's cells; each one em tall at its aspect, in its own colours. |
| `hudMaxAspect` | `number` | `2.3333333` | ≥ 0 | — | — | The widest the HUD band is, over its height: on a wider surface (a 21:9 or 32:9 ultrawide monitor) a HUD document placed at its CSS env(-og-hud-inset-*) (`right: calc(env(-og-hud-inset-right) + 24px)`) lay out in a band this wide, centred in the safe area, so the player's eyes stay near the play; og.ui.viewport's hud is that band. 16/9 keeps a 16:9 game's HUD where it was designed; 0 is the whole safe area (a HUD at the screen's edges). Full-screen layers (dialogs, menus, scrims, world-space markers and their edge clamping) keep the whole surface. The UI itself never stretches: it lays out in CSS pixels at every aspect. |
| `style` | `Style \| null` | `null` | — | — | — | A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the default style (fonts, icons and a primary colour given here still win); null is the default style, the design language the engine shares with origin-agent. |
| `sounds` | `UiSounds \| null` | `null` | — | — | — | Sounds for UI events (they win over the style's). |

```ogt
struct UiDesc {
  doc "How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it."
  aliases "UI scale, text scale, font size, accessibility, high contrast, reduced motion, UI settings"
  desc true
  field scheme Scheme "The palette."
  field density DensityMode "The control sizes."
  field font Option.Font "The body font, from og.load.font; null measures text at half an em per character and draws none."
  field displayFont Option.Font "The display font; null uses the body font."
  field monoFont Option.Font "The monospaced font; null uses the body font."
  field roundedFont Option.Font "The rounded font (levels and scores); null uses the display font, then the body font."
  field fallbackFonts List.Font "Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. A text mixing scripts takes each run from the first font that has it (the role's for its Latin, a fallback for its CJK or Arabic), one label in several fonts."
  field contrast Contrast "Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast)."
  field primaryColor Option.Color "An sRGB colour as the game's primary (its hover, pressed and soft tones derived from it, the ink on it whichever of ink and inkInverse reads better); null keeps the style's."
  field reducedMotion bool "Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion)."
  field textScale f32 "The player's text size (a settings screen's slider, kept with the other settings): every text size and leading is multiplied by it and the layout grows around the text (control, row and header heights by the leading body text gains), never clipping it. 1 is the style's sizes, 1.5 the largest (a larger one lays out at 1.5): docs/ui.md §6.1 holds every screen to 1.5, 200 % deferred." {
    default 1
    min 1
    max 2
    unit ratio
  }
  field unitScale f32 "The og-ui unit in CSS px (docs/ui.md §6.1): the UI lays out in units (og.ui.viewport and og.ui.frameViewport are the surface over it) and draws them at it, so every length grows with the surface (the style's text, space, radii and controls, and a game's own px, alike) and the player's textScale multiplies over it. 0 is auto: on a mouse or pad surface its 1080p-equivalent height (the shorter of the height and the width's 16:9 height) over 720, 1 to 2, so a 1280x720 layout fills a 1080p window with body text at 21 CSS px and the smallest step at 18; a touch surface keeps CSS px (1). A value of 1 to 2 pins it (a game that lays out for one size)." {
    default 0
    min 0
    max 2
    unit ratio
  }
  field reducedTransparency bool "Nothing is see-through: glass parts (a style's glass line) are opaque surfaces (no blur of the frame under them, their frame whole), the scrim is the opaque colour it makes over backdrop and the translucent fills (control, controlStrong, primarySoft, hud) the opaque colours they make over panel. The page's prefers-reduced-transparency turns it on too (Viewport.reducedTransparency); UiTheme.reducedTransparency says it is on."
  field colorVision ColorVision "The player's colour vision (a settings screen's colour-blind mode): the whole frame, the scene and the UI alike, is recoloured for it after every other effect (colorVisionFilter), and a document's CSS reads it as @media (-og-color-vision: protanopia) to add the cues it needs. none (the default) draws the frame as it is. Simulation uses Viénot 1999 for protanopia and deuteranopia and Brettel 1997 for tritanopia; correction is Fidaner 2005's daltonisation over them (docs/ui.md §3.2)."
  field colorVisionFilter ColorVisionFilter "What colorVision does to the frame: correct (the default; the player's setting) or simulate (how a viewer with it sees the frame, for checking a screen)."
  field colorVisionStrength f32 "The deficiency's severity, 0 to 1: 1 (the default) a dichromat's; less blends towards the frame as it is, for the commoner anomalous trichromacy." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field icons Option.IconAtlasDesc "The icon atlas a document's `<og-icon>` draws from; null takes the style's."
  field fallbackIcons Option.IconAtlasDesc "The icon set that draws the names the atlas lacks (icons, or the style's when icons is null): a game's whole set, such as the UI kit's iconoir set, under a style pack's few painted icons, which keep the names they paint. A document's `<og-icon>`, rich text's [icon=], an action's glyph and an art slot's placeholder all look in it; a name in neither warns. null is none."
  field images List.UiInlineImage "Pictures rich text draws inline by name ([img=<name>]): a portrait in a line of dialogue, an item's art in a tooltip. Any texture, not only the icon atlas's cells; each one em tall at its aspect, in its own colours."
  field hudMaxAspect f32 "The widest the HUD band is, over its height: on a wider surface (a 21:9 or 32:9 ultrawide monitor) a HUD document placed at its CSS env(-og-hud-inset-*) (`right: calc(env(-og-hud-inset-right) + 24px)`) lay out in a band this wide, centred in the safe area, so the player's eyes stay near the play; og.ui.viewport's hud is that band. 16/9 keeps a 16:9 game's HUD where it was designed; 0 is the whole safe area (a HUD at the screen's edges). Full-screen layers (dialogs, menus, scrims, world-space markers and their edge clamping) keep the whole surface. The UI itself never stretches: it lays out in CSS pixels at every aspect." {
    default 2.3333333
    min 0
  }
  field style Option.Style "A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the default style (fonts, icons and a primary colour given here still win); null is the default style, the design language the engine shares with origin-agent."
  field sounds Option.UiSounds "Sounds for UI events (they win over the style's)."
}
```

<sub>Module `ui` line 177</sub>

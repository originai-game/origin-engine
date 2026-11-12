# fantasy: style pack

A dark-fantasy RPG dress in standard CSS: warm textured ground, crimson
header bands in letter-spaced small caps, brass double rims with filigree
corners, gold dividers with a centre diamond, painted button plates (crimson
primary, dark iron secondary) and rarity rims that glow over a tinted well.
Load it with `og.load.style(world, "@styles/fantasy")`.

The pack's CSS applies in the `og-pack` layer, above the built-in components'
structure (`og-components`) and below a game's own styles (`og-game`, unlayered).

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face` from `@styles/shared`), the root tokens for the dark and light schemes, the page ground |
| `controls.css` | Every core control's parts and states through `host::part(name):state`: painted button plates by `border-image`, tabs, toggles, sliders, inputs, slots, tooltips and the rest |
| `kit-rpg.css` | The rpg components (`packages/engine/src/ui/components/rpg`) through its root classes and `data-*` hooks: frames, bands, rarity rims, vitals bars, action bar, commands, map, compass, minimap frame |
| `motion.css` | Transitions over a slow warm curve, `-og-spring()` on tab indicators, the acting portrait and fresh badges; a `prefers-reduced-motion` branch keeps the state changes and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library for press, hover, select, confirm, back, open, close, error and reward, with quieter variants on cells and HUD plates |
| `art/` | Frame and button art (SVG), filigree corners, the glow and the ground (PNG), each with its import `.ogt` (colour art sRGB; masks linear) |

## Art provenance

The SVGs were drawn by hand for this pack. `ground.png` and
`filigree-{tl,tr,bl,br}.png` were generated for it with gpt-image-2.6
through the OriginGame gateway (2026-11) and post-processed (cropped,
mirrored, keyed to alpha); `glow.png` is a computed soft radial falloff.
Fonts come from `@styles/shared` under their own licences there.

## Pending engine gaps

Rules written in their correct form are kept and marked `/* pending: … */`
where an engine feature is still landing:

- A heading's `::part(flourish)` is read as a text span and refuses
  `background`, so the gold rule under a heading is not drawn yet
  (parts-as-nodes).
- The `font-family` fallback list is not consulted per glyph: Newsreader
  has no arrows, so a `unicode-range` face takes them from Spectral.

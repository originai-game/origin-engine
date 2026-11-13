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
| `base.css` | Fonts (`@font-face` from `@styles/shared`) and every value the pack draws with, as `:root` tokens for the dark and light schemes: colours, gradients, shadows, lengths, durations, easings, font families, art and sound `url()`s |
| `controls.css` | Every core control's parts and states through `host::part(name):state`: painted button plates by `border-image`, tabs, toggles, sliders, inputs, slots, tooltips and the rest |
| `rpg.css` | The rpg components (`packages/engine/src/ui/components/rpg`) through their named parts (`[part="RpgBackpack.frame"]`, `button::part(RpgBackpack\.cell)`) and `data-*` states: frames, bands, rarity rims, vitals bars, action bar, commands, map, compass; and the rpg sheet's `.rpg-ground`, `.rpg-fold` and `.rpg-minimap` wrappers |
| `motion.css` | Transitions over a slow warm curve, `-og-spring()` on tab indicators, the acting portrait and fresh badges; a `prefers-reduced-motion` branch keeps the state changes and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library for press, hover, select, confirm, back, open, close, error and reward, with quieter variants on cells and HUD plates |
| `art/` | Frame and button art (SVG), filigree corners, the glow and the ground (PNG), each with its import `.ogt` (colour art sRGB; masks linear) |

## Tokens

Only `base.css` writes values; `controls.css`, `motion.css`, `rpg.css` and
`sounds.css` name them with `var()` (`og style check` enforces it), so a
variant restyles the pack by setting tokens alone
(`templates/styles/tests/fantasy-frost`).

- Engine tokens it sets: the palette roles (`--panel`, `--ink`, `--line`,
  `--line-strong`, `--primary`, `--accent`, `--focus`, `--gold`, `--hud`,
  `--scene-*`, `--color-shadow`…), `--font-*`, `--radius-*`,
  `--field-border-width`, `--field-shadow`, `--slot-shadow`,
  `--button-pressed-shadow`, `--dropdown-list-shadow`. `--line-strong` is
  every brass rim and `--accent` every gold wash and glow, so changing one
  restyles all of them.
- Steps of a role: `--accent-N`, `--line-strong-N`, `--shadow-N` are the
  role at N % alpha (`rgb(from var(--accent) r g b / …)`), the engine's
  `--line-70` naming; they follow the role.
- Control-named tokens the engine does not read (it has no image, ink or
  fill token for these): `--button-image*`, `--button-primary-image*`,
  `--button-*-ink`, `--button-font`, `--button-text-shadow`,
  `--button-image-end` (the plate's slice), `--checkbox-checked-fill`,
  `--radio-card-*`, `--switch-*-fill`, `--slider-knob-*`, `--field-fill`,
  `--field-focus-shadow`, `--panel-fill`, `--panel-shadow`, `--slot-fill`,
  `--tabs-indicator-glow`, `--keyboard-key-fill`, `--action-face-*`.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`,
  `--title-*`, `--row-*`, `--cell-*` (and `--cell-<rarity>-*`), `--badge-*`,
  `--well-*`, `--figure-*`, `--label-*`, the state inks, `--trend-*`,
  `--tone-*`, the art marks (`--mark-*`) and sounds (`--sfx-*`). The default
  style dresses every frame, band, title, row and cell from them, so the
  pack has no rule for those roles; it also sets the em type steps
  (`--font-step-*`) and the curve and durations (`--motion-easing-out`,
  `--motion-duration-*`).
- Pack-private `--fantasy-*`, where no engine token means that thing: the
  filigree and ground art, long loops (`--fantasy-motion-glint`,
  `-acting`), and the composite fills and shadows one rpg component alone
  wears (the minimap bezel, the compass strip, party members, the vitals
  rings, command plates…).

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

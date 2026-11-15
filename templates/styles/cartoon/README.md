# cartoon: style pack

A casual cartoon dress in standard CSS, in the spirit of Brawl Stars, Clash
Royale and Fall Guys: bright candy colours on a sky ground, one thick dark
outline around every control, frame and tile, chunky rounded buttons that
stand on a raised bottom edge and sink onto it when pressed, bold rounded
display type, soft drop shadows and springy motion. Load it with
`og.load.style(world, "@styles/cartoon")`, or name it in a look
(`ui "cartoon"`).

The pack is a theme of CSS tokens. It applies in the `og-pack` layer, above
the built-in components' structure (`og-components`) and below a game's own
styles (`og-game`, unlayered), and it selects no component part: every
built-in component and screen wears the default style's control and role
rules, which read these tokens. It has a dark scheme (indigo panels on a
deep sky) and a light one (cream panels on a day sky):
`:root[data-scheme=light]` in `base.css`.

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face`) and every value the pack draws with, as `:root` tokens for the dark and light schemes |
| `motion.css` | Transitions on a spring (`-og-spring()`) for the core controls and the row and cell roles, the pressed plate's drop; a `prefers-reduced-motion` branch keeps every state change and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library, pitched up, with a range so repeated taps differ |
| `art/` | Marks, bursts and the ground's dots (SVG), each with its import `.ogt` (colour art sRGB; masks linear) |
| `LilitaOne-Regular.ttf`, `Fredoka-*.ttf` | The pack's Latin faces, each with its import `.ogt`, and their licences (`OFL-*.txt`) |

## Tokens

Only `base.css` writes values; `motion.css` and `sounds.css` name them with
`var()` (`og style check` enforces it), so a variant restyles the pack by
setting tokens alone.

- The palette roles (`--backdrop`, `--panel`, `--well`, `--ink`, `--line`,
  `--line-strong`, `--primary`, `--selection`, `--accent`, `--focus`,
  `--gold`, `--hud`, `--scene-*`, the tones and teams), the rarity colours,
  `--font-*` and the em type steps, `--radius-*`, the strokes, the motion
  curve, every duration and the spring and scale steps. `--line-strong` is
  the outline around everything and `--selection` the blue of whatever is
  chosen, so changing one restyles all of them.
- The core controls' tokens: `--button-*` (the plate and its lip are
  `--button-*-fill` and `--button-*-shadow`; `--button-pad` and
  `--button-pressed-pad` sink the label with the face), `--checkbox-*`,
  `--radio-*`, `--switch-*`, `--tabs-*`, `--slider-*`, `--field-*`,
  `--dropdown-*`, `--stepper-*`, `--details-*`, `--panel-*`, `--tooltip-*`,
  `--hud-*`, `--slot-*`, `--badge-*` (a toned badge is a plate of its tone:
  `--badge-<tone>-fill` and `-ink`), `--meter-*`, `--keyboard-*`,
  `--touch-*`, `--scrollbar-*`.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`
  (a band is a plate of its own in the outline: `--band-rim`), `--title-*`,
  `--row-*`, `--cell-*` (and `--cell-<rarity>-*`), `--badge-*`, `--well-*`,
  `--figure-*`, `--label-*`, `--name-*`, `--picture-*`, `--tag-*`, the
  state inks, `--trend-*`, `--tone-*`, the art marks (`--mark-*`) and
  sounds (`--sfx-*`).
- One component's own look: `--bubble-tail-*`, `--reticle-*`,
  `--damage-arc-stroke`, `--minimap-bezel`, `--star-*`, `--celebration-*`,
  `--compass-window-*`, and the sheets' `--ground-fill`.
- Pack-private `--cartoon-*`: the candy colours and their lips (a plate is
  its colour as a gradient with the lip under it), the outline and the
  edges (`--cartoon-outline-width`, `-outline-thin`, `--cartoon-lip`,
  `-edge`, `-edge-small`, `-press`, `-drop`, `-ring*`), the text outline and
  drop, the leadings, the sky and the art, and the springs.

A gradient fill ends `padding-box`: a gradient repeats under a border, and
its far end would show along the outline.

Tokens the engine does not read yet are listed with the engine-gaps thread
as missing hooks; until it reads them, that part wears the default shape in
this pack's colours.

## Fonts and art

- Lilita One (Juan Montoreano, 2011), SIL Open Font License 1.1
  (`OFL-LilitaOne.txt`): display, buttons and figures. The file is the
  release from Google Fonts (`ofl/lilitaone`, version 1.002), unmodified.
- Fredoka (The Fredoka Project Authors, 2016), SIL Open Font License 1.1
  (`OFL-Fredoka.txt`): text and labels. `Fredoka-Medium.ttf` and
  `Fredoka-SemiBold.ttf` are instances of the variable font from Google
  Fonts (`ofl/fredoka`, version 2.001) at weights 500 and 600, width 100,
  subset to Latin-1, Latin Extended-A and general punctuation, unhinted.
  Fredoka declares no reserved font name.
- Arrows and any other glyph the two faces lack come from Instrument Sans
  (`@styles/shared`, the `Cartoon Fallback` family after them in every
  font list) and Chinese from ZCOOL KuaiLe (`@styles/cjk`), each under its
  own licence there.
- No face here carries Arabic, so the `ar` locale warns of missing glyphs
  under this pack; a game that ships Arabic adds a face for it.
- The SVGs in `art/` were drawn for this pack; nothing in it is generated.
- Sounds are the shared set (`@styles/shared/sfx-*`).

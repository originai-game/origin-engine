# fantasy: style pack

A dark-fantasy RPG dress in standard CSS: warm textured ground, crimson
header bands in letter-spaced small caps, brass double rims with filigree
corners, gold dividers with a centre diamond, painted button plates (crimson
primary, dark iron secondary) and rarity rims that glow over a tinted well.
Load it with `og.load.style(world, "@styles/fantasy")`.

The pack is a theme of CSS tokens. It applies in the `og-pack` layer, above
the built-in components' structure (`og-components`) and below a game's own
styles (`og-game`, unlayered), and it selects no component part: every
built-in component wears the default style's control and role rules, which
read these tokens.

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face` from `@styles/shared`) and every value the pack draws with, as `:root` tokens for the dark and light schemes |
| `motion.css` | Transitions on the core controls over the pack's slow warm curve, a spring on the tab indicator; a `prefers-reduced-motion` branch keeps the state changes and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library for press, hover, select, confirm, back, open, close, error and reward |
| `art/` | Frame and button art (SVG), filigree corners, the glow and the ground (PNG), each with its import `.ogt` (colour art sRGB; masks linear) |

## Tokens

Only `base.css` writes values; `motion.css` and `sounds.css` name them with
`var()` (`og style check` enforces it), so a variant restyles the pack by
setting tokens alone (`templates/styles/tests/fantasy-frost`).

- The seeds: `--seed-primary` (crimson), `--seed-accent` (brass),
  `--seed-neutral` (walnut), the status seeds and `--seed-team`. The default
  style derives every palette role (`--panel`, `--ink`, `--line`,
  `--primary`, `--accent`, `--focus`, `--hud`, `--scene-*`, the statuses and
  teams) from them in both schemes at the contrast its pairs need, so a
  variant or `og.ui.configure`'s `seeds` retints the pack. Then `--gold`,
  `--color-shadow`, `--font-*`, the em type steps (`--font-step-*`),
  `--radius-*`, and the curve and durations. `--line-strong` is every brass
  rim and `--accent` every gold wash and glow, so changing one restyles all
  of them. `--accent-N`, `--line-strong-N` and `--shadow-N` are the role at
  N % alpha (the engine's `--line-70` naming).
- The core controls' tokens: `--button-*` (the painted plate is
  `--button-image*` with `--button-image-slice` and `--button-image-width`),
  `--checkbox-*`, `--radio-*`, `--switch-*`, `--tabs-*`, `--slider-*`,
  `--field-*`, `--dropdown-*`, `--stepper-*`, `--details-*`, `--panel-*`,
  `--tooltip-*`, `--hud-*`, `--slot-*`, `--meter-*`, `--keyboard-*`,
  `--touch-*`, `--heading-flourish`, `--scrollbar-*`.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`,
  `--title-*`, `--row-*`, `--cell-*` (and `--cell-<rarity>-*`),
  `--badge-*`, `--well-*`, `--figure-*`, `--label-*`, `--tag-*`, the state
  inks, `--trend-*`, `--tone-*`, the art marks (`--mark-*`) and sounds
  (`--sfx-*`). The filigree corners are layers of `--frame-fill` and
  `--plate-fill`, so they need no rule.
- One component's own look: `--bubble-tail-fill`, `--reticle-keyline`, and
  the sheets' `--ground-fill`.
- Pack-private `--fantasy-*`: the art `url()`s and the few values the tokens
  above share.

Tokens the engine does not read yet are listed with the engine-gaps thread
as missing hooks; until it reads them, that part wears the default shape in
this pack's colours.

## Art provenance

The SVGs were drawn by hand for this pack. `ground.png` and
`filigree-{tl,tr,bl,br}.png` were generated for it with gpt-image-2.6
through the OriginGame gateway (2026-11) and post-processed (cropped,
mirrored, keyed to alpha); `glow.png` is a computed soft radial falloff.
Fonts come from `@styles/shared` under their own licences there.

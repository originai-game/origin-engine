# scifi: style pack

A science-fiction instrument dress in standard CSS: dark translucent glass
over the scene, thin cyan rims that glow, chamfered frames with corner
brackets, faint scan lines, condensed technical type, a lit cyan primary
and an amber accent for what is chosen, tracked or focused. Warning is
yellow and danger red, and each keeps its own rim and ink. Motion is quick
and exact: nothing overshoots. It has a dark scheme and a light one
(`og.ui.configure(world, { scheme: "light" })`). Load it with
`og.load.style(world, "@styles/scifi")`.

The pack is a theme of CSS tokens. It applies in the `og-pack` layer, above
the built-in components' structure (`og-components`) and below a game's own
styles (`og-game`, unlayered), and it selects no component part: every
built-in component and screen wears the default style's control and role
rules, which read these tokens. Review them under the pack with
`og ui review <group> --style scifi`, and likewise `components`, `screens`
and `world`.

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face` from `@styles/shared`) and every value the pack draws with, as `:root` tokens for the dark scheme and a `:root[data-scheme=light]` block |
| `motion.css` | Transitions on the core controls over a curve that lands without overshoot, a straight `-og-spring()` on the tab indicator; a `prefers-reduced-motion` branch keeps the state changes and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library for press, hover, select, confirm, back, open, close, error and reward, pitched up and kept short and quiet |
| `art/` | Corner brackets (a dark and a light set), band ends and rule, the ground grid and the map and list marks, all SVG, each with its import `.ogt` (colour art sRGB) |

## Tokens

Only `base.css` writes values; `motion.css` and `sounds.css` name them with
`var()` (`og style check` enforces it), so a variant restyles the pack by
setting tokens alone.

- The seeds: `--seed-primary` (cyan; a deeper teal in the light scheme),
  `--seed-accent` (amber), `--seed-neutral` (deep sea), the status seeds and
  `--seed-team`. The default style derives every palette role from them in
  both schemes at the contrast its pairs need. `--line-strong` stays a
  literal cyan on top, because the derived lines are near-neutral. Then
  `--gold`, `--color-shadow`, the rarity colours, `--font-*`,
  the em type steps, `--radius-*`, the curve and every duration.
  `--line-strong` is every cyan rim, `--primary` the lit face of the main
  action and every glow, and `--accent` every amber wash. `--primary-N`,
  `--accent-N` and `--line-strong-N` are the role at N % alpha.
- The core controls' tokens: `--button-*`, `--checkbox-*`, `--radio-*`,
  `--switch-*`, `--tabs-*`, `--slider-*`, `--field-*`, `--dropdown-*`,
  `--stepper-*`, `--details-*`, `--panel-*`, `--tooltip-*`, `--hud-*`,
  `--slot-*`, `--badge-*`, `--meter-*`, `--keyboard-*`, `--touch-*`,
  `--scrollbar-*`. Each cut corner is a `--scifi-cut-*` radius (two values,
  chamfering the leading top and trailing bottom corners) with its
  `--*-corner-shape` set to `bevel`.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`,
  `--title-*`, `--row-*`, `--cell-*` (and `--cell-<rarity>-*`), `--badge-*`,
  `--well-*`, `--figure-*`, `--label-*`, `--name-*`, `--picture-*`,
  `--tag-*`, the state inks, `--trend-*`, `--tone-*`, the art marks
  (`--mark-*`) and sounds (`--sfx-*`). The corner brackets, the scan lines
  and the glass are layers of `--frame-fill` and `--plate-fill`.
- One component's own look: `--bubble-tail-fill`, `--reticle-*`,
  `--damage-arc-stroke`, `--minimap-bezel`, and the sheets' `--ground-fill`.
- Pack-private `--scifi-*`: the materials every composite is built from
  (`--scifi-glass-*`, `--scifi-plate-*`, the thinner `--scifi-frost-*` for
  what blurs, `--scifi-cell-*`, `--scifi-key-*`, the glow, shade and keyline
  steps, `--scifi-chosen-*`), the cuts, the art and the blurs. The light
  scheme restates the light brand seed, `--line-strong` and these, and
  nothing else.

Two things about the engine shape the values:

- It blends in linear light. A dark glass over a bright scene reads much
  thinner than its alpha, and a bright wash over dark glass much stronger,
  so the washes are low (0.1 and under). The light scheme needs the
  opposite, which is why the wash steps are material tokens.
- Glass that blurs is frost. Large surfaces (frames, panels and dialogs:
  `--frame-glass`, `--panel-glass`) are thin frost (0.70 to 0.77), so the
  blurred scene reads through them as soft colour while their ink keeps
  4.5:1. The HUD's frost (`--plate-glass`, `--hud-glass`,
  `--scifi-frost-plate-*`, 0.88 to 0.91) stays denser, because a HUD stands
  over a sky or a lit wall where its captions still have to read.

Tokens the engine does not read yet are listed with the engine-gaps thread
as missing hooks; until it reads them, that part wears the default shape in
this pack's colours.

## Art provenance

Every SVG in `art/` was drawn by hand for this pack; there are no generated
textures. Fonts (Chakra Petch, Barlow Condensed, Barlow Semi Condensed,
Rajdhani) come from `@styles/shared` under their own licences there.

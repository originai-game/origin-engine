# scifi: style pack

A science-fiction instrument dress in standard CSS: dark translucent glass
over the scene, thin cyan rims that glow, chamfered frames with corner
brackets, faint scan lines, condensed technical type, a lit cyan primary
and an amber accent for what is chosen, tracked or focused. Warning is
yellow and danger red, and each keeps its own rim and ink. Motion is quick
and exact: nothing overshoots. It has a dark scheme and a light one
(`og.ui.configure(world, { scheme: "light" })`). Load it with
`og.load.style(world, "@styles/scifi")`.

The pack's CSS applies in the `og-pack` layer, above the built-in components'
structure (`og-components`) and below a game's own styles (`og-game`, unlayered).

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face` from `@styles/shared`) and every value the pack draws with, as `:root` tokens for the dark scheme and a `:root[data-scheme=light]` block: colours, gradients, shadows, lengths, durations, easings, filters, font families, art and sound `url()`s |
| `controls.css` | Every core control's parts and states through `host::part(name):state`: cut button plates, tabs, toggles, sliders, inputs, slots, badges, tooltips, dialogs and the rest. The file is one cascade layer (`scifi-controls`), see below |
| `rpg.css`, `shooter.css`, `strategy.css`, `racing.css`, `card.css`, `puzzle.css`, `idle.css`, `rhythm.css`, `arena.css`, `general.css` | One file for each built-in component group (`packages/engine/src/ui/components/<group>`), through the components' named parts (`[part="RpgBackpack.frame"]`, `button::part(RpgBackpack\.cell)`) and `data-*` states; `rpg.css` also dresses the rpg sheet's `.rpg-ground`, `.rpg-fold` and `.rpg-minimap` wrappers |
| `components.css` | The ungrouped components (`packages/engine/src/ui/components/*.svelte`) where the role dress is not enough: the hero button's faces, a menu's quiet rows, the data grid's headings on its lit band, the loot label's tag on the scene plate, a nameplate's cast bar, a status effect's time on the scene plate, a radial menu's wedges and item buttons, and the speech bubble's tail, a shape, in one colour of the glass |
| `motion.css` | Transitions over a curve that lands without overshoot, a straight `-og-spring()` on tab indicators, the scan loops on the acting portrait and fresh badges; a `prefers-reduced-motion` branch keeps the state changes and drops the movement |
| `sounds.css` | `-og-sound` sets from the shared library for press, hover, select, confirm, back, open, close, error and reward, pitched up and kept short and quiet |
| `art/` | Corner brackets (a dark and a light set), band ends and rule, the ground grid and the map and list marks, all SVG, each with its import `.ogt` (colour art sRGB) |

The other ungrouped components and the built-in screens have no rule of
their own: they are made of core controls and role parts, which
`controls.css` and the role tokens dress. Review them under the pack with
`og ui review components --style scifi`, and likewise `screens` and
`world`.

## Tokens

Only `base.css` writes values; every other file names them with `var()`
(`og style check` enforces it), so a variant restyles the pack by setting
tokens alone.

- Engine tokens it sets: the palette roles (`--panel`, `--ink`, `--line`,
  `--line-strong`, `--primary`, `--accent`, `--focus`, `--gold`, `--hud`,
  `--warn`, `--danger`, `--scene-*`, `--color-shadow`…), the rarity colours
  (`--common` to `--mythic`), `--font-*`, `--radius-*`, the control
  surfaces (`--button-*` with `--button-selected-*`, `--field-*`,
  `--slot-*`, `--switch-*`, `--slider-knob-*`, `--radio-card-*`…), the
  curve and every duration (`--motion-easing-out`, `--motion-duration-*`
  from `instant` to the long loops `turn`, `pulse`, `sweep`, `breathe`,
  `sheen`, `drift`, `fanfare`, `orbit`) and `--opacity-wash`.
  `--line-strong` is every cyan rim, `--primary` the lit face of the main
  action and every glow, and `--accent` every amber wash, so changing one
  restyles all of them.
- Steps of a role: `--primary-N`, `--accent-N`, `--line-strong-N` are the
  role at N % alpha (`rgb(from var(--accent) r g b / …)`); they follow the
  role.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`,
  `--title-*`, `--row-*`, `--cell-*` (and `--cell-<rarity>-*`), `--badge-*`,
  `--well-*`, `--figure-*`, `--label-*`, the state inks, `--trend-*`,
  `--tone-*`, the art marks (`--mark-*`) and sounds (`--sfx-*`). The default
  style dresses every frame, band, title, row and cell from them, so the
  pack has no rule for those roles; their cut corners are
  `--frame-corner-shape`, `--plate-corner-shape` and `--cell-corner-shape`
  (`bevel`) on the `--scifi-cut-*` radii. The corner brackets, the scan lines and the glass are layers of `--frame-fill` and
  `--plate-fill`.
- Pack-private `--scifi-*`, where no engine token means that thing:
  - the materials every composite is built from (`--scifi-glass-*`,
    `--scifi-plate-*`, their thinner `--scifi-frost-*` for what blurs,
    `--scifi-cell-*`, `--scifi-key-*`, the glow, shade
    and keyline steps, `--scifi-chosen-*`). The light scheme restates the
    palette, the inks and these, and nothing else;
  - the cuts (`--scifi-cut-*`, a two-value radius that `corner-shape: bevel`
    turns into a chamfer on the top-left and bottom-right corners, and
    `--scifi-oct`, the octagon of a portrait, seal or medal);
  - the art (`--scifi-bracket-*`, `--scifi-grid`, `--scifi-scanlines`), the
    dimming filters (`--scifi-dim-*`), the loops (`--scifi-motion-*`), and
    the composite fills and shadows one component alone wears.

Two things about the engine shape the values:

- It blends in linear light. A dark glass over a bright scene reads much
  thinner than its alpha (and lets bright text under it ghost through), and
  a bright wash over dark glass much stronger, so glass that does not blur
  is nearly opaque (0.95 and up) and the washes are low (0.1 and under).
  The light scheme needs the opposite, which is why the wash steps are
  material tokens.
- Glass that blurs is frost: the backdrop blur removes the detail that
  would ghost through, so frost is thinner (0.78 to 0.84) and the scene
  shows through it as soft colour. The HUD's frost (`--scifi-frost-plate-*`,
  0.88 to 0.91) stays denser, because a HUD stands over a sky or a lit wall
  where its captions still have to read. `--scifi-glass-blur` (panels,
  dialogs, tooltips and every role frame) and the lighter
  `--scifi-plate-blur` (HUD plates: `.hud` frames, which wear the HUD's
  frost, `hud`, the shooter's vitals and ammunition, the rpg compass) are
  the blurs. A bare frame, a frame inside another (it
  stands on glass that already blurred), world tags, per-value bars, the
  small chips of a busy HUD and trays its slots cover do not blur: they
  wear the solid `--scifi-frame-solid-fill` and `--scifi-plate-solid-fill`
  instead, since each blur is a capture of its own.
- A pack's files apply in lexical order, so `arena.css` and `card.css` are
  read before `controls.css`. A component button that names its own part
  still matches `button::part(base)`, and at equal specificity the later
  file would win. `controls.css` is therefore one cascade layer: a rule that
  names a component's part outranks a core control's rule wherever its file
  falls.

## Art provenance

Every SVG in `art/` was drawn by hand for this pack; there are no generated
textures. Fonts (Chakra Petch, Barlow Condensed, Barlow Semi Condensed,
Rajdhani) come from `@styles/shared` under their own licences there.

## Pending engine gaps

- A core `panel` and `dialog` draw an opaque surface under their frame
  part, so their frost and blur do not show: the scene stays hidden even
  with the fill at 0.25. Role frames, HUD plates and the compass blur.
- The focus part of a button whose document names its own part also takes
  `button::part(base)`'s rules, so a plate is drawn over the button's
  content on keyboard focus. `controls.css` clears `button::part(focus)` to
  the ring alone.
- Pack files apply in name order and a named button part still matches
  `button::part(base)`, so a group file sorted before `controls.css` loses
  to it. `controls.css` is one cascade layer for that reason.

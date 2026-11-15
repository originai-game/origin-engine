# cartoon: style pack

A casual cartoon dress in standard CSS, in the spirit of Brawl Stars, Clash
Royale and Fall Guys: bright candy colours on a sky ground, one thick dark
outline around every control, frame and tile, chunky rounded buttons that
stand on a raised bottom edge and sink onto it when pressed, bold rounded
display type, soft drop shadows and springy motion. Load it with
`og.load.style(world, "@styles/cartoon")`, or name it in a look
(`ui "cartoon"`).

The pack's CSS applies in the `og-pack` layer, above the built-in components'
structure (`og-components`) and below a game's own styles (`og-game`, unlayered).
It has a dark scheme (indigo panels on a deep sky) and a light one (cream
panels on a day sky): `:root[data-scheme=light]` in `base.css`.

| File | What it holds |
| --- | --- |
| `base.css` | Fonts (`@font-face`) and every value the pack draws with, as `:root` tokens for the dark and light schemes: colours, gradients, shadows, lengths, durations, easings, font families, art and sound `url()`s |
| `controls.css` | Every core control's parts and states through `host::part(name):state`: the button plates and their lips, tabs, toggles, sliders, fields, select, stepper, slots, tooltips, dialogs, touch controls and the rest; and the role parts the role tokens do not carry. The file is one cascade layer (`cartoon-controls`), see below |
| `motion.css` | Transitions on a spring (`-og-spring()`), the pressed plate's drop, the loops (a waiting reward wobbles, the current level bobs, a fresh badge throbs); a `prefers-reduced-motion` branch keeps every state change and drops the movement |
| `rpg.css`, `shooter.css`, `strategy.css`, `racing.css`, `card.css`, `puzzle.css`, `idle.css`, `rhythm.css`, `arena.css`, `general.css` | One file for each built-in component group (`packages/engine/src/ui/components/<group>`), through its named parts and `data-*` states |
| `components.css` | The ungrouped components (`packages/engine/src/ui/components/*.svelte`) where the role tokens and the core controls leave one undressed: the hero and hold buttons' plates, the radial menu's wedges, the table's cells, the tooltip card's hints |
| `sounds.css` | `-og-sound` sets from the shared library, pitched up, with a range so repeated taps differ |
| `art/` | Marks, bursts and the ground's dots (SVG), each with its import `.ogt` (colour art sRGB; masks linear) |
| `LilitaOne-Regular.ttf`, `Fredoka-*.ttf` | The pack's Latin faces, each with its import `.ogt`, and their licences (`OFL-*.txt`) |

The built-in screens and most ungrouped components need no rule of their
own: their frames, bands, titles, rows, cells, badges and figures are role
parts, dressed by the default style from this pack's role tokens, and their
buttons, tabs and fields are the core controls.

A pack's files apply in name order, so `arena.css`, `card.css` and
`components.css` are read before `controls.css`. `controls.css` is therefore
one cascade layer: a rule that names a component's part outranks a core
control's rule wherever its file falls.

## Tokens

Only `base.css` writes values; every other file names them with `var()`
(`og style check` enforces it), so a variant restyles the pack by setting
tokens alone.

- Engine tokens it sets: the palette roles (`--backdrop`, `--panel`, `--well`,
  `--ink`, `--line`, `--line-strong`, `--primary`, `--selection`, `--accent`,
  `--focus`, `--gold`, `--hud`, `--scene-*`, the tones and teams), the rarity
  colours (`--common`, `--uncommon`, `--rare`, `--epic`, `--legendary`,
  `--mythic`), `--font-*` and the em type steps, `--radius-*`, the strokes,
  the motion curve and every duration (`--motion-duration-instant` to
  `-orbit`), the spring and scale steps, `--opacity-wash`, and the control
  tokens the engine reads (`--button-pressed-shadow`,
  `--button-selected-fill`, `-ink`, `-lip` and `-shadow`, `--field-*`,
  `--slot-shadow`, `--dropdown-list-shadow`). `--line-strong` is the outline
  around everything and `--selection` the blue of whatever is chosen, so
  changing one restyles all of them.
- Control-named tokens the engine does not read: `--button-fill`,
  `--button-*-fill`, `--button-*-shadow`, `--button-*-ink`, `--button-font`,
  `--button-pad` and `--button-pressed-pad` (the label sinks with the face),
  `--checkbox-*`, `--switch-*`, `--slider-knob-*`, `--field-fill`,
  `--field-focus-shadow`, `--keyboard-key-fill`.
- The role tokens (docs/ui.md §3.1): `--frame-*`, `--plate-*`, `--band-*`,
  `--title-*`, `--row-*`, `--cell-*` (and `--cell-<rarity>-*`), `--badge-*`,
  `--well-*`, `--figure-*`, `--label-*`, the state inks, `--trend-*`,
  `--tone-*`, the art marks (`--mark-*`) and sounds (`--sfx-*`).
- Pack-private `--cartoon-*`, where no engine token means that thing:
  - the candy colours and their lips (`--cartoon-yellow`, `-blue`, `-green`,
    `-red`, `-orange`, `-pink`, `-purple`, `-royal`, `-slate`, `-white`,
    `-grey`, `-silver`, each with `-lip`): a plate is its colour as a
    gradient with the lip under it;
  - the outline and the edges (`--cartoon-outline-width`, `-outline-thin`,
    `--cartoon-lip`, `-edge`, `-edge-small`, `-press`, `-drop`,
    `-plate-shadow`, `-ring*`, `-glow`): the dark outline dropped below a
    plate is what makes it stand up;
  - the text outline and drop (`--cartoon-text-outline`, `-text-drop`,
    `-stroke-*`), the leadings, the ground (`--cartoon-sky`, `-ground`,
    `-dots-image`), the art (`--cartoon-burst-image`, `-rays-image`,
    `-star-image`) and the loops (`--cartoon-spring`, `-spring-soft`,
    `-motion-wobble`, `-motion-shine`, `-wobble-*`, `-bob`, `-throb`,
    `-swell`).

A gradient fill ends `padding-box`: a gradient repeats under a border, and
its far end would show along the outline.

## Buttons

The default button reads with `--ink`, as content written for a control
expects: a violet plate with white ink in the dark scheme, a white plate with
dark ink in the light one. `.primary` is yellow with `--on-primary`,
`.danger` red, `.ghost` an outline, `.quiet` bare, and a button that is on
(`aria-pressed`) is the selection's blue.

`button::part(base)` also matches a button that names its own part (a tile,
a row, a tab), so the plate's rules select `button:not([part])::part(base)`
and each group's file draws its own named buttons. The type and the resting
ink are on the host, so a button's content inherits them.

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
- Arrows come from Instrument Sans (`@styles/shared`) and Chinese from ZCOOL
  KuaiLe (`@styles/cjk`), each under its own licence there.
- The SVGs in `art/` were drawn for this pack; nothing in it is generated.
- Sounds are the shared set (`@styles/shared/sfx-*`).

## Pending engine gaps

Parts and states the pack could not reach through tokens and parts. None is
worked around in CSS; the scifi pack's README lists the same ones where it
met them.

- In a sheet that sets `og.ui.configure(world, { scheme: "light" })` with a
  pack loaded, the pack's `:root[data-scheme=light]` block is not applied
  (fantasy and scifi show the same): the frames stay dark while a few inks
  turn dark with the engine's own theme. The light values here were judged
  with that block promoted to the default in a scratch copy.
- An `og-icon` multiplies its picture by `color`, glyph or full-colour art
  alike. Under the light scheme's dark ink a full-colour item picture (the
  rpg sheet's backpack and equipment cells) is drawn as a dark silhouette.
- A button's plate is reached only through `button::part(…)`, and in that
  selector `.secondary` is also every button without a `variant`. A hero
  button's `secondary` tone cannot be told from its default, so both wear
  the yellow plate.
- The radial menu's item button names no part and clears its plate in the
  component's own layer, which the pack's plate rule outranks, so the core
  plate is drawn on each wedge.
- A `kbd` prompt inside a button (action hints, the hero button's key, the
  sheets' Back, Equip, Lock in and Preview) is drawn as an empty cap.
- A node graph's node covers part of a neighbouring label (the card run
  map; under fantasy too).
- The slider bubble's text is always `--ink-inverse` and the checkbox glyph
  always `--on-primary`: neither part takes an ink, so their plates are
  chosen to suit those roles.
- A modal `dialog` (the confirm screen) is drawn as a
  flat surface: `dialog::part(panel)` reaches a dialog opened in place but
  not one in the top layer, so it has no outline or lip (fantasy too).
- A core `badge`'s label is drawn in the engine's own face: `font-family`
  on `badge::part(base)` does not reach it, while its plate and ink do.
- A celebration's word and value sit in a row in the component's own
  stage, which names no part, so the level reads as a suffix of the word.
- `select::part(option)` has no checked state, so the chosen option in an
  open list looks like the others; `input::part(frame)` has no hover.
- The default style dresses every `[part$=".cell"]` as a tile, readouts
  included (a lap count, a score, a wave number, a table's cell): each
  file restyles those cells by name.
- The `font-family` fallback list is not consulted per glyph: neither face
  has arrows, so a `unicode-range` face takes them from Instrument Sans,
  once for each weight a family declares.
- The review's contrast measure reads text over a gradient or a layered
  ground wrongly (a white title on the blue band measures 1.5 to 2.4 there
  and 11.9 with `og frame measure` on the same rect); flagged text was
  checked against the pixels.
- There is no review sheet for the ungrouped components or the built-in
  screens; they were judged in a scratch project from their first fixture.
  The world-anchored components (action hints, damage numbers, interaction
  prompt, loot label, nameplate, overhead bars, path line, placement ghost,
  selection ring, speech bubble, telegraph) had no world point on screen
  there and have not been seen in a frame under this pack.
- Lilita One and Fredoka carry no Arabic, so the `ar` locale warns of
  missing glyphs under this pack; no Arabic face is shipped with it.

# debt/style-names: what the ui/style round left owed (2026-10-23)

Branch `debt/style-names` from main b80aa70c, an orb (Linux, lavapipe),
so every Metal reference is owed. The fourth item of the list, og-cli's
doctor tests, was already on main (c4d613c5) and was not touched.

## What landed

| Commit | What |
|---|---|
| d948de18 | og-ui: the skin identifiers take style names (table below), in og-ui, og-cook, og-runtime, og-cli and their tests; READMEs follow. A refactor: og-ui's 113 tests pass on lavapipe, every reference frame unmoved; og-runtime's ui_configure and missing_glyphs and og-cli's style pack test pass; clippy clean |
| aa3372a1 | og-ui: a style file's two errors say style (`unknown style setting`, `the style has an atlas but no slice lines`); the one assertion that read the first follows. The `type skin` refusal keeps the old word on purpose: it names what to change |
| 99ef7117 | ui-kit: the `skin:` command is `style:` (state field `style`, `STYLES`; nothing of the old word is left in the command set), the playtests `skin-<pack>[-phone]` are `style-<pack>[-phone]` built by `tests/styles.ts` (`stylePlaytest`), the twelve Metal sheets `kit-skin-*` are `kit-style-*` (git mv, bytes unchanged), `make-skins.py` is `make-styles.py`; reference-ui.ts and reference.ts (the web tiers), docs/ci.md, og-ui's README, the og-ui skill, every style's header and LICENSES line that named the script, meridian-loop's and idle's tools follow. `make-styles.py` rerun writes every `style.ogt` as committed (its `icons.png` differ by a few texels of at most 16/255 from the committed ones, a Pillow version's resampling, so they were left as committed) |
| a95108bf | tower-defense: Path Guard on the current cartoon art (below) |
| f1fe7663 | rumpus-isle: the rumpus style follows the cartoon pack again (below) |
| 9c431fc1 | tower-defense: pathguard's primary darkened for the new button art; README wording |
| 87f3c784 | api.ogt: og.load.look named the look's UI line `skin` (a UI skin's asset id), but the key is `ui`; og.render.applyLook "configured with its skin" now says style. Doc wording only; `og api gen`, `og api gen --wiki`, `og api check` rerun (the d.ts, the schema, wiki load.md and render.md) |
| this one | these notes, the before/after crops beside them, the two games' READMEs |

No `og test`, `scripts/check.sh`, web-reference or release build was run.
The coordinator's instruction after the infrastructure stop had this
thread run `OG_GATE=1 sh scripts/build.sh` in packages/engine to refresh
the stale native addon. With no gate outputs in the orb, it also built
the wasm and ran wasm-opt (unmeasured, nothing committed from it).

## Renames (item 1)

| Old | New | Why |
|---|---|---|
| `crates/og-ui/src/skin.rs`, `og_ui::skin` | `src/style_file.rs`, `og_ui::style_file` | the module parses and holds the style file (`style.rs` was already the theme's types) |
| `Skin` (re-exported `og_ui::Skin`) | `Art` | the drawn part of a style: atlas, nine-slices, ornaments, shadows, glows, text effects, press and hover, motion, sounds |
| `SkinMotion` | `StyleMotion` | |
| `Theme::skin` | `Theme::art` | |
| `Element::skin`, `PanelDesc::skin` (an element's nine-slice part) | `Element::frame`, `PanelDesc::frame` | the same word as the IDL's `frame` field it is read from |
| `FontRole::skin_index` | `FontRole::effect_index` | the index of a role's text shadow, glow and weight |
| `StyleFile::resolve(skin_id, …)`, `resolve_reference(skin_id, …)` | `style_id` | |
| og-runtime `skin_part`, `skin_font`, locals `skin` | `frame_part`, `style_font`, `style` / `style_name` | |
| `crates/og-ui/tests/skin.rs` and its `SKIN`, `skinned_ui`, test names | `tests/style.rs`, `STYLE`, `styled_ui`, `a_styled_panel_…`, `…_draws_no_atlas_pieces`, `a_style_file_names_…`, `a_style_resolves_…`, `…_when_the_style_says_repeat_…` | |
| pack_contrast `skins()`, `game_skins()`, `every_skin_keeps_…` | `styles()`, `game_styles()`, `every_style_keeps_…` | |
| og-runtime ui_configure `a_skin_names_its_assets_…` | `a_style_names_its_assets_…` | |
| og-cli `skin_fixture` | `style_fixture` | |
| og-cook README `og.UiSkin` / `SkinMeta`; og-cli README `og skin` | `og.UiStyle` / `StyleMeta`; `og style` | the code already had the new names |
| ui-kit `skin:`, state `skin`, `SKINS` | `style:`, `style`, `STYLES` | |
| ui-kit `tests/skin-<pack>[-phone].og.ts`, `tests/skins.ts`, `skinPlaytest` | `tests/style-<pack>[-phone].og.ts`, `tests/styles.ts`, `stylePlaytest` | |
| `crates/og-ui/references/metal/kit-skin-<pack>[-phone].png` | `kit-style-<pack>[-phone].png` | |
| `templates/ui-kit/tools/make-skins.py` (`skin_text`) | `make-styles.py` (`style_text`) | |
| rumpus-isle `tools/make-assets.ts` `skin()` | `style()` | |

The glTF meaning of skin (og-anim, og-render's `skin.rs`, og-cook's
`SkinRow`, physics' character `skin`) is a different word and stays.

## Frames read (lavapipe, `og run --headless --commands`)

- **ui-kit**: `0.2:style:cartoon` dresses the gallery in cartoon
  (`dump_ui` reports `style: "cartoon"`). The whole `style-cartoon`
  playtest script through `og run`: each of the 21 screens, box-filtered
  as styles.ts does, against its cell of the moved
  `kit-style-cartoon.png` (Metal): 0.000 % outliers each, RMSE at most
  0.0016. The sheets moved by name only.
- **Path Guard** (templates/archetypes/tower-defense; `rg` confirms it
  is not under examples/), `tests/frames.og.ts`'s title, wave and end
  against the Metal references: 0.34 to 0.49 % outliers, all at the UI.
  Its style kept a byte-for-byte copy of the cartoon atlas before
  07fab232, radius 10/18 and insets 20 to 30, and its HUD passed
  `radius: 999`. Now the HUD boxes (lives, coins, wave, hint) are boxes
  with 6 px corners, not pills; the tray's cards are 10 px; the title
  and end panels' corners are modest; the Play, Settings, Play again and
  tray buttons are rounded rectangles, not capsules. Play and the
  selected plant's button are a deeper rust (`#974009`, was `#b44c0d`):
  the new button art is lighter on average, and pack_contrast read the
  primary label at 5.54:1 against its 7:1 bar until the primary moved
  (og-ui pack_contrast and packs: 4 of 4 pass). The garden, the
  visitors and every text are unchanged.
  `pathguard-{hud,tray,title}-before-after.png`: old Metal frame left,
  new lavapipe right (hud and tray before the primary moved).
- **Rumpus Isle**: its generator already copies the current cartoon
  atlas (gitignored), but the committed `style.ogt` still had the old
  radii and insets, so the new art drew under the old insets. On
  lavapipe, before (committed files) against after, the 11 frames of
  `tests/frames.og.ts` move 0.06 to 0.57 %, all in the UI. The lobby's
  seat cards go from 18 to 10 px corners; the scoreboard's rows from 14
  to 6. The pause panel, its buttons, the round box and the HUD boxes
  draw the pack's frames at their proportions (with the 20 px insets
  they were thick-edged and near-square). The island, the critters and
  every text are unchanged. Against Metal the frames differ 0.4 to
  3.1 %; the lavapipe before against Metal already differs 0.55 to
  0.64 %, so the rest is the tier.
  `rumpus-{pause,lobby}-before-after.png`: lavapipe before left, after
  right.

## Metal references owed (the coordinator's)

- **By name only**: none to record. The twelve `kit-style-*` sheets were
  moved with git mv and keep their bytes, and the lavapipe check above
  matches them. The web tiers have no references of their own (they
  judge against metal), so none moved.
- **By content**: `templates/archetypes/tower-defense/references/metal/`
  `title.png`, `wave.png`, `end.png`; `examples/rumpus-isle/references/metal/`
  all eleven (`title`, `lobby`, `card`, `bump`, `splash`, `scores`,
  `berry`, `crumble`, `pause`, `settings-fr`, `podium`). Their browser
  tiers go to the wave's end.

## Left out, and why

- **"skin" in prose elsewhere**: the READMEs, LICENSES headers and
  comments of the other games (brasswick, hollowbell, lantern-vale,
  kestrel-stair, meridian-loop, the idle, rhythm, roguelike and
  third-person archetypes) still call their style a "skin", and some
  still name the old `skins/<name>` paths. That is wording the games
  own, not a renamed identifier, so it is outside these three items. A
  follow-up can sweep it. docs/phase-1.md and the CHANGELOG are history
  and keep the old names.
- **The `type skin` refusal** in `parse_style` keeps the old word: it
  exists to name the fix for an old file.
- **Seen, not fixed**: Rumpus Isle's small `Pause` button in the round
  HUD (about 22 px tall, under the cartoon button's 2 × 12 px corners)
  shows a faint light band at its left and right ends on lavapipe after
  the change. It is the engine's nine-slice at a size below its corners
  (the pack's art, any game using `size: "small"` buttons), not this
  game's art. A stray small red mark beside "Paused" and "Pip rules
  Rumpus Isle!" is in the before frames too (likely the heading flourish; not investigated).
- **pathguard's `icons.png`** was already identical to the pack's and
  stays; its LilitaOne and Fredoka copies are unchanged.

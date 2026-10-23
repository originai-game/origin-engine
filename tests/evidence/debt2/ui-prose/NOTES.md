# debt2/ui-prose: og-ui's nine-slice band and the games' "skin" prose (2026-10-23)

Branch `debt2/ui-prose` from main b4694e67, in an orb (Linux, lavapipe),
so every Metal reference is owed. No `og test`, `scripts/check.sh`,
web-reference, wasm or release build was run. The loop was `cargo
nextest run -p og-ui`, `cargo clippy -p og-ui`, the knowledge package's
`bun test`, `og api check`, and the two scripts below through `og run`.

## What landed

| Commit | What |
|---|---|
| 613e684d | og-ui: a nine-slice's middle never squeezes its art more than twice, with the regression test |
| 6e4d7f85 | knowledge: the skills and README say style where they meant og-ui's style |
| 71f382b2 | knowledge: `validate` refuses a skill or wiki page that calls a UI style a skin, and a test for it |
| e9d964b7 | games: the skin prose and the old `skins/<name>` paths (the file list below) |
| 4adcd6a5 | og-ui: the middle's cut leaves the corners' cut alone (a fix to 613e684d; the test now checks it) |
| this one | these notes and the before/after crops beside them |

## Item 1: the band was not the corners

The brief, following debt/style-names' NOTES, put the band down to the
corners drawn at full size on an element shorter than them. They were
not: `nine()` has scaled the corners to fit both axes independently since
d612bc39, and the Pause button is 64 × 28 px, above the cartoon button's
2 × 12 px corners (`dump_ui`: `Pause` at 727, 19, 64 × 28). The real cause
was the middle. Its 24-texel middle row was drawn in 4 px, a 6× squeeze.
The atlas is mipmapped (og-cook's `to_texture_data`, `Mips::Generate`)
and the GPU picks one mip level per pixel from the larger of the two
axes, so the edge pieces read level 2 to 3 and blurred the dark border
into the cream fill across both ends. That is the light band.

The fix: when a middle column or row would squeeze its art more than
twice, it shows the centre `2 × len` of the art's middle instead of all of
it. The corners keep their own cut. The bound is 2, not 1, so an element
whose middle is at least half its art's is unchanged. That covers every
medium (44 px) and phone (40 px) control in every pack. At 1, every 44 px
button of every pack would move, because a 48-texel art makes a 1.2×
squeeze. At 2, the outline through the middle rows is continuous but one
level softer (mip level 1) than the corner rows (below).

The test `a_frame_smaller_than_its_art_tiles_exactly_and_never_squeezes_its_middle_past_twice`
(crates/og-ui/tests/style.rs) checks three panels against a 48-texel
slice with 12-texel corners: 64 × 28 (the Pause case), 16 × 10 (smaller
than both corner pairs) and 64 × 60. For each, the pieces tile the
element exactly (the areas sum, no two overlap), and no middle piece
squeezes its art more than twice. Each corner piece also shows exactly
its 12 texels. On b4694e67 it fails with "64x28: … squeezes its middle
art 1x6". On 613e684d it fails with "shows 12x20 texels of its corner":
that first version moved the cut points the corners share, and the
Rumpus frames showed soft, thin ends, which is how it was found. og-ui:
114 tests pass, clippy is clean.

## Frames read (lavapipe, `og run --headless`)

`og run` loads the published 0.3.0 addon from bun's cache unless
`OG_NATIVE_ADDON` names another, so a checkout's change needs the dev
addon (`cargo build -p og-host-node`, copied to a `.node` name). Before
is the dev addon built from b4694e67's og-ui, byte-identical in every
frame to the published 0.3.0 addon. After is 4adcd6a5.

- **Rumpus Isle**: the whole `tests/frames.og.ts` script (the same
  steps, printed with `tests/lib.ts`' `script`), 1280 × 720.
  - title and scores are unmoved.
  - card, bump, splash, berry, crumble and pause move 200 to 204 px, all
    inside the Pause button's 4 middle rows (y 31 to 34).
  - Before, the outline's columns in those rows read 101 to 134 grey,
    against 39 above and below: the light band at both ends.
  - After, they read 23 and 98, then the fill. The dark outline runs
    unbroken from the top corners to the bottom ones, with no band.
  - The fill's own highlight line across the middle is the art's (it is
    in the before frame across the whole width).
  - lobby moves 2,633 px: the colour chips (Coral, Sun, Mint disabled,
    Sky, Grape, Cream) are small buttons with the same band, now gone.
  - settings-fr moves 617 px: the Pause button and the small `Terminé`.
  - podium moves 3,524 px: the middle rows of Play again, New players
    and Title.
  - Nothing else moves. Crops: `rumpus-pause-before-after.png` (8×, the
    bump frame), `rumpus-lobby-before-after.png`,
    `rumpus-settings-podium-before-after.png`.
- **ui-kit, the cartoon pack** (`style-cartoon`'s script, from
  `tests/script.ts`' `framesScript(0.5)` after `0.2:style:cartoon`):
  20 of 21 screens move, by 3 to 8,449 px. All of it is in the middle rows
  of controls under 36 px (the gallery's small Back, the toasts, the
  shop's and results' small buttons); cutscene and loading are unmoved.
  So the expectation that styles stay unmoved where elements are larger
  than their corners does not hold: the band's case is an element larger
  than its corners, and every pack draws one. Against the committed Metal
  `kit-style-cartoon.png` under styles.ts' rule, before and after both
  pass 21/21 cells (after: at most 0.010 % outliers, RMSE 0.0034; before:
  0.000 %, 0.0017).

## Metal references owed (the coordinator's)

Moved by content, at small controls' middle rows only:

- `examples/rumpus-isle/references/metal/`: `lobby`, `card`, `bump`,
  `splash`, `berry`, `crumble`, `pause`, `settings-fr`, `podium` (`title`
  and `scores` unmoved).
- `crates/og-ui/references/metal/kit-style-{cartoon,fantasy,paper,pixel,scifi}.png`.
  The arithmetic: a 28 px control's middle is under half its art's in
  those five, while neon's 2 × 16 px button corners are larger than 28 px,
  so its middle is 0 either way. Cartoon was measured; the other four were
  not run. The `-phone` sheets (40 px small controls) and the
  default-theme `kit-desktop-*`, `kit-phone-*` and `kit-locale-*` (no
  atlas) should not move.
- Any other game frame showing a styled control under twice its corners
  plus half its art. In the repository only ui-kit and Rumpus Isle pass
  `size: "small"`; not swept further.

## Item 2: the skin prose

Swept (`rg -n -i skin`, each hit read; 82 files in e9d964b7):

- **brasswick**: README, LICENSES.md, src/ui.ts, prefs.ts, main.ts,
  tests/locales.og.ts, tools/make-assets.ts (comments, the `skin()` step
  now `style()`, the header string it writes), looks/brasswick.ogt,
  styles/brasswick/LICENSES.md and style.ogt (the header the generator
  writes, kept equal). README's `skins/brasswick` is `styles/brasswick`.
- **hollowbell**: README (`skins/hollowbell` is `styles/hollowbell`),
  FINDINGS #34's summary, src/ui.ts, components.ts, tools/subset-font.py,
  looks/hollowbell.ogt, styles/hollowbell/LICENSES.md and style.ogt.
- **lantern-vale**: README, FINDINGS #6 (`skins/fantasy/...` is
  `styles/fantasy/...`; the finding kept), #11 (`look(world)?.skin` is
  `?.style`, which is the field's name now), and the fixed entries for #6
  and #11. Also src/ui.ts, components.ts, tools/subset-font.py,
  tests/locales.og.ts, styles/lantern/LICENSES.md and style.ogt.
- **kestrel-stair**: README, src/ui.ts, components.ts,
  tools/subset-font.py, tools/make-assets.ts (the `style()` step, the
  `style.ogt` error, and the header and LICENSES strings it writes),
  tests/locales.og.ts, looks/kestrel.ogt, styles/kestrel/LICENSES.md and
  style.ogt.
- **meridian-loop**: README, LICENSES.md, src/ui.ts, tools/make-assets.ts
  (comments, `const style`, the header string),
  styles/meridian/LICENSES.md and style.ogt.
- **rumpus-isle**: FINDINGS #2 (`Skin()` is `Style()`, the handle's name,
  og.Style), #10 (`og-ui/tests/skin.rs` is `tests/style.rs`), the display
  shadow entry and the cook summary. Also LICENSES.md, src/prefs.ts,
  main.ts, tests/locales.og.ts, looks/rumpus-isle.ogt and
  styles/rumpus/LICENSES.md.
- **The archetypes idle, rhythm, roguelike and third-person**: README,
  src/ui.ts, main.ts (`const skin` is `const style`, passed as
  `{ style, … }`), components.ts (`leftTitle`'s doc), sound.ts
  (third-person), idle's tools/make-assets.py and looks/idle.ogt, and each
  style's LICENSES.md and style.ogt header.
- **templates/archetypes/README.md**, and the "skin's `fallback`" line of
  every `templates/styles/*/LICENSES.md`.
- **packages/knowledge** (6e4d7f85): og-feel, og-build, og-platform-sdk,
  og-i18n (with its `skins/neon/...` path) and og-audio; the README's
  og-ui summary.

The check (71f382b2): `validate` splits each skill and wiki page (the
payload under `game/skills/`) into sentences and table cells. A sentence
that says `skin` or `skins` must also name one of the word's other
meanings: glTF, joint, skeleton, morph, rig, bone, mesh, model, clip,
anim, the `` `skin` `` field, surface, gap, a skin tone, peach, crowd or
tiles. Otherwise it is "calls a UI style a skin". The committed package
passes (43 tests). The new test adds a UI use to og-ui's skill and to
wiki/physics.md beside allowed ones, and fails without the check. Run on
the skills before the sweep, the check names all 13 swept sentences.

api.ogt says "skin" only for glTF skins, the controllers' collision
`skin`, peach skin and texture variants (`tiles, skins, damage states`),
so it was not edited and the wiki was not regenerated. `og api check`:
the generated files match. The knowledge README is not in the payload, so
the check does not read it; it was swept by hand.

## Left out, and why

- **The other meanings stay.** These keep "skin":
  - glTF skins and skinning, in kestrel's rig.ts, gltf.ts, foes.ts and
    hero.ts, lantern-vale's slime, meridian's gltf.ts and fx-showcase's
    README (all four of its hits are skinned models, so nothing there
    changed). The same goes for third-person's hero.ts and
    og-game/og-core/og-build.
  - The controllers' collision skin: hollowbell's world.ts and FINDINGS
    #15 and #16, and the wiki's physics pages.
  - Character skin colours in hollowbell's and lantern-vale's
    make-assets, and the runner and topdown archetypes.
  - "The orb's skin" in idle's drift effect, "skin tone" in og-ui's
    skill and in the catalogue, and the word inside "asking"
    (meridian-loop FINDINGS #17, og-taste).
- **Stream names in history**: `skins-palette` in kestrel-stair FINDINGS
  #15 and hollowbell FINDINGS #12 names a past stream, not a style.
- **Outside the brief's list**, still saying skin for a UI style:
  - The tower-defense archetype: src/ui.ts lines 1 and 354,
    components.ts' `leftTitle` doc, main.ts' `const skin` and
    `style: skin`.
  - packages/engine's tests/look.test.ts (lines 7 and 89) and its
    fixture `looks/base.ogt` (line 4).
  - Each is the same one-line sweep.
- **History**: docs/phase-1.md, CHANGELOG.md and tests/evidence, the
  `type skin` refusal in og-ui's `parse_style`, og-cook and og-render were
  not touched.
- **The 1× bound** (crisper outline through the middle rows, at the cost
  of moving every 44 px button in every pack) is the coordinator's
  decision.
- **The release 0.3.0 addon and a dev build** were compared on these
  frames and draw them identically, so the published engine has the same
  band.

## Shared files touched

- `crates/og-ui/src/ui.rs` (`nine`, the new `middle_cut`) and
  `crates/og-ui/tests/style.rs` (a test, and `panel_pieces` now calls
  `sliced_pieces`).
- `packages/knowledge/scripts/validate.ts` (`skinWord`, called from
  `validate`) and `packages/knowledge/tests/validate.test.ts`.
- api.ogt is untouched, so the records-list stream's api.ogt edits do not
  meet these.

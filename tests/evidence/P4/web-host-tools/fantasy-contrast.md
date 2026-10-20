# The fantasy pack's labels at 4.5:1, Ember Vale's touch HUD on the ui bus (owed row 48)

2026-10-20, Apple M4 Pro, metal, the dev addon (frame times in the polish
reports are a loaded machine's and the debug build's; read the taste rows).

## What og polish found

Ember Vale (`templates/archetypes/third-person`, the fantasy pack copied as
`assets/skins/ember`), `og polish` before (`ember-polish/report-before.md`,
with the HUD fix below already in): Set out 3.36:1, the title's footer
3.5:1, Again 3.99:1, Resume 3.84:1, 完成 4.41:1, 语言 4.43:1; on the phone
1 of 3 UI presses without a ui-bus sound (2 of 3 before this round).

`contrast-tools/run.sh` reproduces polish's measure (median luminance of the
element's box as the ground, the 1 % of its pixels farthest from it as the
ink; `contrast-tools/measure.py`) on the three shots it flags, so a palette
could be tried in 20 s instead of a full polish run:

| Label | ink token | ground | measured before | after |
|---|---|---|---|---|
| Set out (primary) | accentFg on accent | the button art × accent | 3.46 | 6.56 |
| Settings (secondary) | fg on fill | the button art × fill | 4.47 | 4.75 |
| Footer | fgSubtle on the dialog | parchment | 3.50 | 5.51 |
| Resume (primary) | accentFg on accent | | 3.89 | 6.85 |
| 语言 (LanguageLabel) | fgMuted, Noto Sans SC | parchment | 4.38 | 5.50 |
| 完成 (Done, primary) | accentFg on accent | | 4.41 | 7.06 |
| Again (primary) | accentFg on accent | | 4.09 | 7.11 |

Why the frame reads lower than the tokens: the 13 to 15 px labels are thin
MSDF strokes, so their darkest pixels never reach the ink (Set out's ink
token is luminance 0.0125, its darkest percent measured 0.046), and the
button's art shades its tint by about a fifth (the atlas centre is 0.9
sRGB). On paper Set out was 5.3:1 and the footer's fgSubtle 3.5:1.

## The palette

Both `templates/ui-kit/assets/skins/fantasy/skin.ogt` (and its generator,
`templates/ui-kit/tools/make-skins.py`) and Ember Vale's copy:

| Token | before | after | why |
|---|---|---|---|
| accent | #c99a3a | #7a520f | the primary fill: a dark bronze under a cream label instead of mid gold under a dark one (a mid tone cannot carry dark ink at 4.5:1 once drawn, and a pale gold fill left the accent-tinted icons invisible on parchment) |
| accentHover | #d8ac4c | #74500e | the hover art is lighter (0.95 sRGB), so the hover tint is a shade darker to keep its label |
| accentActive | #b0832a | #6a470c | |
| accentFg | #2a1a0c | #fff4dc | the cream label |
| accentSoft | #c99a3a40 | #7a520f40 | follows accent |
| fill | #d8c08d | #e2cc98 | secondary buttons, fields, slots: lighter under fg |
| fillStrong | #c9ab6f | #d6bc82 | |
| fgMuted | #5c4127 | #3d2a18 | 语言 and other muted labels |
| fgSubtle | #8a6d4c | #6a4f33 | the footer: 3.5:1 on paper before |
| danger | #9b2a1c | #6e1b10 | the danger button's label on its tint: 4.02:1 on paper before |

The type ramp is unchanged (meta and caption at 14 px changed nothing the
measure reads; the footer's ink was fgSubtle, not the size).

After (`ember-polish/report.md`): **contrast 100** (lowest 4.65:1, the
secondary Settings button; Set out 6.56 by the reproduction), **UI sounds
100** (phone 3 of 3), taste 100.

## The test

`crates/og-ui/tests/pack_contrast.rs` is `button_contrast.rs` for the
packs: every role's label in every state and the three label inks, as og-ui
draws them with a pack's skin, against what is drawn under them (the draw
commands composited in linear light, a skinned piece as the mean of its
atlas texels times its tint). For the fantasy pack and Ember Vale's copy a
primary or secondary label at rest or hovered must reach 7:1 there (the
frame reads about a third lower: Set out 5.3 there, 3.36 in polish) and
every other label 4.5:1 on a panel. With the old fantasy palette it fails
on ten rows: the primary label at rest (5.32:1) and pressed (3.64:1), the
hovered secondary (6.87:1), on the screen and on a panel; the danger label
at rest, hovered and pressed (4.02, 4.15, 3.31); fgSubtle on a panel
(3.49). `pack-contrast-table.txt` has the after table for all seven packs.

The other five packs are printed, not held: on a panel, cartoon's primary
label is 3.35:1 at rest and its danger label 1.75:1, paper's and
cartoon's fgSubtle 3.22 and 3.28, pixel's 4.18, neon's pressed secondary
3.60, paper's pressed danger 4.15, pixel's hovered danger 4.03. They are
another stream's packs this round and are left for it.

## The HUD

Ember Vale's touch HUD answered only on the sfx bus (the jump's and the
fire's own sounds). A `Tap` voice plays the skin's press sound on the ui
bus when a thumb lands on the stick, when Kindle is pressed and when Jump
is released (`src/sound.ts`, `src/ui.ts`); og-ui's touch controls play no
press sound of their own. `og run` with the phone playtest's commands:
the tap voice plays twice by tick 282 (Jump, Kindle).

## References

`og test templates/archetypes/third-person` failed `frames` on the new
palette (title 1.12 % outliers, RMSE 0.0211; dawn 0.55 %, 0.0146; beacon
passed at 0.0088): re-recorded on metal (`OG_FX_CAPTURE=1`). Title: Set
out is a bronze bar with a cream label, Settings a lighter parchment bar,
the footer darker. Dawn: Again bronze with a cream label, the fire icon
bronze instead of gold. Beacon: the HUD's fire icon and the clock's meter
bronze. The WebGL2 beacon, which keeps its own reference, read RMSE
0.01000 against it afterwards (the HUD), so it was re-recorded too
(`tests/web/reference.ts --webgl --record --shots beacon@204`); WebGPU and
WebGL2 title and dawn pass against metal (RMSE 0.0013, 0.0007; 0.0043,
0.0018). ui-kit: `skin-fantasy` and `skin-fantasy-phone` failed against
their sheets and were recorded on metal (`OG_UI_CAPTURE=1 og test
templates/ui-kit`, only those two sheets kept); the other 27 suites pass
unchanged. vulkan-cpu: see the section below.

## vulkan-cpu

The two fantasy sheets re-recorded in the `og-lavapipe` image (llvmpipe,
linux/amd64 under emulation, with bun copied in from `og-test:bun143`),
the addon from `cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31 -p og-host-node` as
`packages/engine/native/origin-engine.linux-x64.node`, and only the two
playtests run (`OG_UI_CAPTURE=1 bun packages/engine/src/node/playtest-run.ts
--file templates/ui-kit/tests/skin-fantasy[-phone].og.ts --entry
templates/ui-kit/.og/game.native.js --root templates/ui-kit`; 220 s and
628 s). Each sheet moved from its old vulkan-cpu reference by RMSE 0.0167
(desktop) and 0.0334 (phone), and sits within RMSE 0.0010 and 0.0011 of
the new metal sheet, as the old pair did. Ember Vale has no vulkan-cpu
references (owed row 47).

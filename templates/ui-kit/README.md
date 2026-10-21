# UI kit demo

Every `og.ui` screen template and the game widget set on one gallery
(`crates/og-ui`, `packages/engine/src/ui-kit`, the IDL module `ui`). Pick a
screen from the list, or `show:<screen>` on the command channel; Back
(Escape, gamepad B, the Gallery button) returns.

```sh
og run templates/ui-kit                       # the window, desktop size
og dev templates/ui-kit                       # the browser, reloading on save
og test templates/ui-kit                      # the playtests below
OG_UI_CAPTURE=1 og test templates/ui-kit      # re-record this tier's reference frames
python3 templates/ui-kit/tools/make-assets.py templates/ui-kit/assets   # regenerate icons and backdrop
python3 templates/ui-kit/tools/make-skins.py templates/ui-kit/assets    # regenerate the six style packs and UI blips
```

Commands: `show:<screen>`, `viewport:<desktop|tablet|phone|landscape>`
(lays the UI out for that surface through `og.ui.previewViewport`; render at
`--size 1170x2532` for the 390×844@3 phone), `pad:<up|down|left|right|next|prev|activate|back>`
(a gamepad, through `og.ui.navigate`), `toast:<text>`, `damage:<hp>`, `heal:<hp>`.
`skin:<tokens|pixel|cartoon|fantasy|scifi|paper|neon>` dresses every screen
in a style pack (`assets/skins/<pack>/skin.ogt` through `og.load.skin`),
`transition:<none|fade|wipe|iris|dissolve|slide>` sends screen changes through
`og.ui.screenTransition`, `sounds:<on|off>` (on by default) hands `assets/audio/ui-*.wav` to
the UI's event sounds, `offline:<on|off>` makes the leaderboard's rows fail
to load (its error message and Retry, which loads again). `dump_entity:demo` reads the demo's state (the `Demo` component).

Screens: gallery, widgets, mainMenu, pause, settings, platformer, shooter,
puzzle, idle, racing, cutscene, shop, inventory, levels, results,
leaderboard, tutorial, loading, confirm, alert, toasts, players (a
local-multiplayer lobby: `og.ui.setPlayers(4)`), text (type and scripts:
the display face kerned, the rounded face for a score, labels cut with an
ellipsis, Arabic in a Latin line and inside a rich-text span, Hebrew,
Devanagari, a text field and a text area). `screens/<screen>.png`
shows each at desktop (left) and on a phone in portrait (right).

Faces: the body is Noto Sans SC; titles take `font: "display"` (Inter at
its display optical size, semibold: `assets/fonts/Inter-Display.ttf`) and
scores `font: "rounded"` (Nunito bold), both cooked as their own atlases
and made by `tools/make-faces.py` from the platform's own copies; Hebrew
and Devanagari subsets are `fallbackFonts`.

## What it demonstrates, and the skills it exercises

The engine's UI as a catalogue rather than a game: every screen template and
widget (og-ui §2, §3), focus by keyboard, gamepad and several fingers at once
(og-ui §4), phones, tablets and the safe area with every screen captured on a
390×844@3 phone (og-ui §5), the six style packs with their tiled nine-slices,
count-ups, bursts, screen transitions and UI sounds (og-ui §6; og-taste
§1–§7; og-audio §3), string tables in English, Chinese, Japanese (kinsoku line
breaking) and Arabic (right to left) (og-i18n §1–§4, §6), and how to prove UI
from a script with `move_to_text`, `dump_ui` and reference frames (og-ui §8;
og-build §1, §3). `og polish .` (2026-10-19): taste 90; feel 62, which reads
a catalogue as a game: its playtests press through screens and capture them
still, its `hp` fact is the meter demo's number, not damage, and keyboard and
gamepad focus moves make no sound (og-ui plays none for a focus move yet).


## The reference source

The demo is where `crates/og-ui/references/<tier>/kit-desktop-<screen>.png`
and `kit-phone-<screen>.png` come from: `tests/frames-desktop.og.ts` and
`tests/frames-phone.og.ts` show each screen, wait for its entrance, take its
frame and compare it with the reference under og-tests' rule (a channel off
by more than 32/255 is an outlier; fail above 0.5 % outliers or RMSE 0.01).
Every frame, locale screen and skin sheet is committed for `metal`,
`vulkan-cpu`, `webgpu` and `webgl2`; a tier without its own references
compares with `metal` and says so. The browser tiers run these same
playtests through `packages/engine/tests/web/reference-ui.ts` (`docs/ci.md`
§Browser reference tiers).

| Playtest | Proves |
|---|---|
| `frames-desktop`, `frames-phone` | every screen matches its reference frame at 1280×720 and at 390×844@3 with a notch and a home indicator |
| `touch` | `move_to_text:=shop` opens the shop from the scrolled gallery, `Detail/Buy` spends the price and toasts, Escape closes; Share calls the share hook; Tap counts; a generator buys; the dialogue box finishes, then advances a line |
| `keys` | Tab, Tab, Enter opens the second row; in Settings, Down reaches the Master slider, Left twice steps it to 0.9, Escape closes and the save hook writes `og.settings` once through og.platform |
| `settings-saved` | across a relaunch (`store: "keep"`): Mute and 简体中文 on the settings screen, Done writes `og.settings`; the relaunch asks for `og.settings` on its first tick and comes back muted, with the main menu in Chinese (`src/main.ts` `followSettings`) |
| `pad` | gamepad intents: down ×4 and A open Settings, B closes; Tab order reaches the select, A opens its menu, down and A choose Hard |
| `a11y` | every screen's accessibility tree (`dump_ui`'s role, name, value and states; `og.ui.a11y()`): every control keys and pads reach has a role and a name, every element showing text has a role, Tab walks the focusable elements in the tree's order and wraps, and `og.ui.a11y()` counts the same nodes and focus order as the probe |
| `hit` | the touch HUD's Hit is felt on its tick: a 3-tick hit stop, camera trauma, the number floating up from the health bar and a thud on the sfx bus (`assets/audio/hit.ogt`, og sfx's hit preset; og-feel §3, §4, §7) |
| `players` | `og.ui.setPlayers(4)`: pads 1 and 2 (`gamepad:right@1`…) move rings of their own from the hero nearest the centre and pick Mage and Rogue for P2 and P3 with A, a touch picks Thief for P1, a tap on the hint's rich-text link (`Hint/join`) follows it, and the frame with both rings matches `kit-desktop-players.png` |
| `refuse` | the Crown (2,400) with 1,250 coins: Buy reads "Not enough", a press shakes it three times back to rest with the error sound on the ui bus (`og.ui.shake`), spends nothing and reports `refused` |
| `drag` | a press on slot 0 dragged to slot 7 moves the potion; Use runs for it |
| `phone` | on a phone the HUD corners sit in the safe area, every touch control is at least 44 CSS px, a tap on Jump answers once |
| `tutorial` | the spotlight's hole contains each step's target, a press on the dimmed part changes nothing, Next walks to Done |
| `skin-<pack>` (six) | every screen but the gallery, in that style pack at 1280×720, matches its 640×360 cell of `crates/og-ui/references/<tier>/kit-skin-<pack>.png` (7 × 3 cells in `FRAME_SCREENS` order) |
| `skin-<pack>-phone` (six) | the same on a phone in portrait (390×844@3 with the notch and home indicator), each screen box-filtered to its 390×844 CSS pixels against its cell of `kit-skin-<pack>-phone.png` (10 × 2 cells) |
| `juice` | an iris transition keeps the old screen until it has closed and builds the new one as it opens; the results score counts and lands on 12,840; a dialog's open, a hover and a press start voices on the ui bus |
| `text` | the text screen draws every script without a missing glyph (no warning), matching `kit-desktop-text.png`; a tap on the name field and `type:Ada Lovelace` fill it once, Shift+← ×4 selects "lace" (`kit-desktop-text-select.png`); in the notes area Enter breaks the line, `compose:にほん` shows underlined at the caret (`kit-desktop-text-compose.png`) and `commit:日本` lands in the value |

## Style packs

`assets/skins/<pack>/` holds a skin (`skin.ogt`), its atlas and icons (CC0,
drawn by `tools/make-skins.py`) and its OFL fonts (`LICENSES.md`). Copy a
pack's directory into a game's `assets/skins/`, keep
`fonts/NotoSansSC-Regular-subset.ttf` for the CJK fallback, and configure:
`og.ui.configure(world, { skin: og.load.skin(world, "skins/fantasy/skin.ogt").value })`.

| Pack | Look |
|---|---|
| `pixel` | retro: square frames with cut corners, 2 px outlines and bevels, hard drop shadows, DotGothic16 |
| `cartoon` | casual: cream panels, thick dark outlines, glossy capsules, squash on press, pops, Lilita One and Fredoka |
| `fantasy` | parchment panels, gold filigree corners and flourishes, Cinzel display over Newsreader |
| `scifi` | HUD: hairline frames with cut corners, corner brackets, scanlines, cyan glow, Orbitron, Chakra Petch, JetBrains Mono numerals |
| `paper` | minimal: off-white sheets, hand-drawn ink outlines, soft shadows, a vermilion accent, Newsreader and Instrument Sans |
| `neon` | arcade: dark glass, glowing magenta and cyan tubes, italic Chakra Petch display, Orbitron numerals |

## Locales

The main menu, pause and confirm screens, the settings screen, the shop,
the inventory, the leaderboard, the tutorial and the cutscene (the
templates' own words through their descs' label fields: `settings.*`,
`shop.*`, `inventory.*`, `leaderboard.*`, `tutorial.*`, `cutscene.*`; the
tutorial's `"{step} of {count}"` filled by the template) and the
settings' language list read their text
from `og.i18n` (`assets/strings/{en,zh-CN,ja,ko,ar}.ogt`,
`type strings`): `og.i18n.t(world, "menu.play")`, with plurals and
arguments in ICU form (`{n, plural, other {コイン#枚}}`). Each table names
its font chain (`fonts "fonts/NotoSansJP-subset.otf"`); the active locale's
fonts come first in og-ui's fallback chain. Arabic is shaped from the
font's own tables (its import settings use the `arabic` charset, which
cooks the joining forms and the outlines) and the layout mirrors right to
left. The language setting calls `og.i18n.setLocale`; `og run --locale ja`
and the browser's `navigator.language` pick the first locale. `og cook`
warns about any key a table lacks.

The main menu's greeting (`menu.greeting`) is drawn with characters no
locale font cooked in zh-CN and ja (欢迎回来，旅人晨星, おかえりなさい、旅人さん): the
Chinese font cooks with `runtime true`, so og-ui renders them from its
outlines when the screen shows them (crates/og-ui/README.md §Text). The
locales playtest asserts no warning, and a frame that draws a missing glyph
warns.

The confirm dialog's message is rich-text markup in each table
(`"…slot [b]{slot}[/b] will be lost. [color=danger]This cannot be
undone.[/color]"`): every translation bolds the slot and colours the
warning in its own word order, Arabic included (`og.ui.richText`; the
dialogs' messages are markup). The players lobby's hint is rich text too,
with a `[link=join]` the `players` playtest taps.

`tools/make-locale-fonts.py <NotoSansCJK-Regular.ttc> <NotoSansArabic.ttf>`
subsets the Noto fonts to exactly the tables' characters (rerun it after
changing a string; the sources are noto-cjk's `Sans/OTC/NotoSansCJK-Regular.ttc`
2.004 and NotoSansArabic v2.012's `googlefonts/variable/NotoSansArabic[wdth,wght].ttf`,
which reproduce the committed subsets but for `head.modified`), leaving the greeting out of zh-CN's and ja's and
`menu.mixed` out of every locale's. Its subsets hold the kit's strings only;
a game makes its own from a whole face with `tools/subset-font.py <source
font> <out font> [--locale <tag>] [--cjk-only]` (og-i18n §3), copied into
the game's `tools/`: it reads the game's tables, writes the subset and its
import settings, and names every character the source lacks. `tests/locales.og.ts` compares each of
the three screens in each locale with
`crates/og-ui/references/<tier>/kit-locale-*`.

`mixed:on` adds `menu.mixed` under those three screens: one label of CJK,
Latin and Arabic (`原点探险 Origin Quest مغامرة صغيرة` in zh-CN), its
characters all in the tables' other strings, so each run comes from a face
that already cooks it: og-ui lays it out a run per face, a grapheme
cluster at a time. `tests/locales-mixed.og.ts` shows it on the three
screens in all five locales and asserts no warning, so `missing_glyphs`
stays 0 on every frame (the engine warns on the first frame that draws a
character its fonts lack); its frames are not references.

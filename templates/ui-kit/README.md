# UI kit demo

The `og.ui` components, the genre patterns and the genre showcases on one
gallery (`crates/og-ui`, `packages/engine/src/ui`, the IDL module `ui`). A
showcase is a genre's screens as a game builds them: ejected screen
documents (`assets/ui/*.ogt`, from `og ui eject`) with the kit's chrome in
their `Kit` slot, and HUDs built from components over the genre scenes
(`src/showcases.ts`, `src/huds.ts`). Pick a
screen from the list, or `show:<screen>` on the command channel; Back
(Escape, gamepad B, the Gallery button) returns.

```sh
og run templates/ui-kit                       # the window, desktop size
og dev templates/ui-kit                       # the browser, reloading on save
og test templates/ui-kit                      # the playtests below
OG_UI_CAPTURE=1 og test templates/ui-kit      # re-record this tier's reference frames
python3 templates/ui-kit/tools/make-assets.py templates/ui-kit/assets   # regenerate icons and backdrop
python3 templates/ui-kit/tools/make-styles.py templates/ui-kit/assets    # regenerate the six built-in styles (templates/styles/) and UI blips
```

Commands: `show:<screen>`, `viewport:<desktop|tablet|phone|landscape>`
(lays the UI out for that surface through `og.ui.previewViewport`; render at
`--size 1170x2532` for the 390×844@3 phone), `pad:<up|down|left|right|next|prev|activate|back>`
(a gamepad, through `og.ui.navigate`), `toast:<text>`, `damage:<hp>`, `heal:<hp>`.
`style:<default|pixel|cartoon|fantasy|scifi|gilded|storybook|tactical|cyber|kawaii|craft|dread|broadcast|wasteland|foundry>` dresses every screen
in the engine's placeholder or a built-in style (`assets/looks/kit-<style>.ogt`,
whose `ui "<style>"` the cook resolves to `@styles/<style>/style.ogt`),
Each style draws on its own backdrop (`STYLE_BACKDROPS` in `src/main.ts`:
key art or a scene from `assets/textures/art/scenes`, the default style
keeping the neutral dusk), and the shop, inventory and results show the kit's
painted art (`assets/textures/art`): items, the merchant Brann, the paperdoll's
Aria Vell with her attributes. `transition:<none|fade|wipe|iris|dissolve|slide>` sends screen changes through
`og.ui.screenTransition`, `sounds:<on|off>` (on by default) hands `assets/audio/ui-*.wav` to
the UI's event sounds, `sc:<reset|tutorial:<n>|hud:<key>:<value>|race:<event>>` sets a
showcase's state (`sc:race:countdown`, `checkpoint`, `wrongWay`, `finalLap`, `finish`: a race's moment), `pat:<area>:<key>:<value>` a pattern preview's. `dump_entity:demo` reads the demo's state (the `Demo` component).

Screens: the showcases first (actionTitle, actionHud, actionPause,
actionTutorial, actionLoading, actionResults, shooterHud, shooterScore,
manageHud, raceHud, raceResults, raceBoard, sportsHud, matchFacts; the
racing HUD draws over an engine-rendered circuit, `src/raceScene.ts`, lit by
its pack: night, broadcast's golden hour, the arcade packs' noon), then gallery,
widgets, the component sheets, the genre patterns (store, gear, saga,
dialogue, vn, lobby, select, garage, ledger, rts, deck, settingsTree and
the rest; `SCREENS` in `src/main.ts`), toasts, players (a
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

The engine's UI as a catalogue rather than a game: the screen documents,
components and patterns (og-ui §2, §3), focus by keyboard, gamepad and several fingers at once
(og-ui §4), phones, tablets and the safe area with every screen captured on a
390×844@3 phone (og-ui §5), the six style packs with their tiled nine-slices,
count-ups, bursts, screen transitions and UI sounds (og-ui §6; og-taste
§1–§7; og-audio §3), string tables in English, Chinese, Japanese (kinsoku line
breaking) and Arabic (right to left) (og-i18n §1–§4, §6), and how to prove UI
from a script with `move_to_text`, `dump_ui` and reference frames (og-ui §8;
og-build §1, §3). Keyboard and gamepad focus moves make no sound (og-ui
plays none for a focus move yet).


## The reference source

The demo is where `crates/og-ui/references/<tier>/kit-desktop-<screen>.png`
and `kit-phone-<screen>.png` come from: `tests/frames-desktop.og.ts` and
`tests/frames-phone.og.ts` show each screen, wait for its entrance, take its
frame and compare it with the reference under og-tests' rule (a channel off
by more than 32/255 is an outlier; fail above 0.5 % outliers or RMSE 0.01).
Every playtest draws its UI over the pack's plain ground (`demo:ground:pack`,
set by the kit's `playtest` in `tests/lib.ts`), never over the key art,
painted scenes or 3D circuit the demo shows: a UI frame is judged over a
plain, solid-colour ground, and a scene by its own tests (docs/ui.md
§15.4). Every frame, locale screen and states sheet is committed for
`metal`; a tier without its own references compares with `metal` and says
so. The
browser tier, WebGPU, runs these same
playtests through `packages/engine/tests/web/reference-ui.ts` (`docs/ci.md`
§Browser reference tiers).

| Playtest | Proves |
|---|---|
| `frames-desktop`, `frames-phone` | every screen matches its reference frame at 1280×720 and at 390×844@3 with a notch and a home indicator |
| `touch` | the store opens by its row's text from the scrolled gallery, Buy spends the price and marks the offer owned, Escape closes; the action results' Play again leads into the HUD; a generator buys; the visual novel's box finishes, then advances a line |
| `keys` | keyboard focus opens a screen, steps a slider on the settings tree to a pending change and closes it |
| `settings-saved` | across a relaunch (`store: "keep"`): master 0, render scale 75 % and 简体中文 on the settings tree, Apply writes `og.settings`; the relaunch asks for it on its first tick and comes back muted, the title in Chinese and the tree's values where Apply left them (`src/main.ts` `followSettings`) |
| `pad` | gamepad intents: down ×4 and A open Settings, B closes; Tab order reaches the select, A opens its menu, down and A choose Hard |
| `a11y` | every screen's accessibility tree (`dump_ui`'s role, name, value and states; `og.ui.a11y()`): every control keys and pads reach has a role and a name, every element showing text has a role, Tab walks the focusable elements in the tree's order and wraps, and `og.ui.a11y()` counts the same nodes and focus order as the probe |
| `hit` | the touch HUD's Hit is felt on its tick: a 3-tick hit stop, camera trauma, the number floating up from the health bar and a thud on the sfx bus (`assets/audio/hit.ogt`, og sfx's hit preset; og-feel §3, §4, §7) |
| `players` | `og.ui.setPlayers(4)`: pads 1 and 2 (`gamepad:right@1`…) move rings of their own from the hero nearest the centre and pick Mage and Rogue for P2 and P3 with A, a touch picks Thief for P1, a tap on the hint's rich-text link (`Hint/join`) follows it, and the frame with both rings matches `kit-desktop-players.png` |
| `refuse` | the Crown (2,400) with 1,250 coins: Buy reads "Not enough", a press shakes it three times back to rest with the error sound on the ui bus (`og.ui.shake`), spends nothing and reports `refused` |
| `drag` | a press on slot 0 dragged to slot 7 moves the potion; Use runs for it |
| `phone` | on a phone the HUD corners sit in the safe area, every touch control is at least 44 CSS px, a tap on Jump answers once |
| `tutorial` | the action tutorial's card sits under each step's target and Next walks to the end |
| `ui-check` | `og ui check` finds no undeclared overlap on the action and shooter HUDs on a phone |
| `style-<pack>` (fantasy, scifi) | the pack's approved states (`STYLE_STATES`: the selected bag cells at 100 and 150 % text), each over the pack's plain ground, match their cells of `crates/og-ui/references/<tier>/kit-style-<pack>-states.png`; a pack's own pages are `og ui sheet`'s (docs/ui.md §15.4) |
| `style-<pack>-phone` (fantasy, scifi, storybook) | the same on a phone in portrait (`PHONE_STYLE_STATES`: the horror HUD at 100 and 150 % text) against `kit-style-<pack>-phone-states.png` |
| `juice` | an iris transition switches screens at full cover; UI events start voices on the ui bus |
| `text` | the text screen draws every script without a missing glyph (no warning), matching `kit-desktop-text.png`; a tap on the name field and `type:Ada Lovelace` fill it once, Shift+← ×4 selects "lace" (`kit-desktop-text-select.png`); in the notes area Enter breaks the line, `compose:にほん` shows underlined at the caret (`kit-desktop-text-compose.png`) and `commit:日本` lands in the value |

## Built-in styles

`templates/styles/<style>/` holds a style (`style.ogt`, complete: every
palette role and radius), its atlas and icons (CC0, drawn by
`tools/make-styles.py`) and its OFL fonts (`LICENSES.md`). A game names one
in its look (`ui "fantasy"`), or copies the directory into its own
`assets/styles/<name>/` to change it and names that (`ui
"styles/<name>/style.ogt"`); a game that shows CJK adds a `fallback` font.
Without one, a game's UI draws in the engine's default style, for
prototypes only (`og build --publish` refuses it).

| Style | Look |
|---|---|
| `pixel` | retro: square frames with cut corners, 2 px outlines and bevels, hard drop shadows, DotGothic16 |
| `cartoon` | casual: cream panels, thick dark outlines, glossy buttons with modest corners, squash on press, pops, Lilita One and Fredoka |
| `fantasy` | parchment panels, gold filigree corners and flourishes, Cinzel display over Newsreader |
| `scifi` | HUD: hairline frames with cut corners, corner brackets, scanlines, cyan glow, Orbitron, Chakra Petch, JetBrains Mono numerals |

The seed styles `og style new` generated (gilded, storybook, tactical,
cyber, kawaii, craft, dread, broadcast, wasteland, foundry) draw from tokens and procedural
dressings with the fonts and icons of `templates/styles/shared/`. The
paper, neon, bubbly, arcade, inkwash and holo packs were folded into
storybook, cyber, cartoon, pixel, jade and scifi (the owner, 2026-11-02).

## Locales

The showcases' documents and HUDs, the patterns and the settings tree (the
`screen.*`, `sc.*`, `set.*` and the patterns' keys; the documents' words
are string keys, the components' through their descs' label fields) and the
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
`crates/og-ui/references/<tier>/kit-locale-*`, and reads the five HUDs in
each locale on a desktop and on a tablet (touch controls shown): in zh-CN,
ja, ko and ar none shows a Latin word but the ability key `Q` and ja's and
ko's `km/h`.

`mixed:on` adds `menu.mixed` under those three screens: one label of CJK,
Latin and Arabic (`原点探险 Origin Quest مغامرة صغيرة` in zh-CN), its
characters all in the tables' other strings, so each run comes from a face
that already cooks it: og-ui lays it out a run per face, a grapheme
cluster at a time. `tests/locales-mixed.og.ts` shows it on the three
screens in all five locales and asserts no warning, so `missing_glyphs`
stays 0 on every frame (the engine warns on the first frame that draws a
character its fonts lack); its frames are not references.

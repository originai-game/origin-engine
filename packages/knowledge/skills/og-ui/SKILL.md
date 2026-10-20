---
name: og-ui
description: Game screens and HUDs on Origin Engine with og.ui — which screen template to pick (title, pause, settings, five HUDs, cutscene, shop, inventory, level select, results, leaderboard, tutorial, loading, dialogs, toasts), how to feed it data and read what it answers, the widget set beneath, focus for keyboard and gamepad, accessibility (the screen-reader mirror), phones, touch gestures and the safe area, text (fields, text areas, selection and the clipboard, input methods, ellipsis, kerning, shaping, font fallback), and the look: skins (style packs) and how their frames tile, count-ups, bursts, shakes, screen transitions and UI sounds. Use when building or changing menus, HUDs or any UI in a project with a project.ogt.
---

`og.ui` is the engine's immediate-mode UI: a system describes the tree every
tick, the engine lays it out, draws it in the platform's design tokens and
answers the probe (`dump_ui`, `touch_move_to`, `move_to_text`). The API
reference is `reference/ui.md`; this skill is how to use it well.

## 1. The rule of the kit

Every template and widget is **one call per tick, data in, answer out**:

```ts
const r = og.ui.shopScreen("Shop", { balance: run.coins, items: SHOP, categories: ["Gear", "Items"] });
if (r.bought !== null) { /* take the price, grant the item */ }
if (r.closed) state.screen = "map";
```

- Keep game state (open flags, the current line, the settings, slots) in a
  component or resource, pass it in every tick, store what comes back. Never
  keep UI state in module variables. A field's or an area's text is game
  state too: keep it in a `str(capacity)` field (og-core §4), four bytes per
  character of the widget's `maxChars` (`name: str(64)` for `maxChars: 16`),
  so the text always fits.
- What only the UI needs between ticks (hover, focus, scroll, a menu being
  open, the chosen tab of a template, animation progress, toasts) is UI state:
  snapshots and replays carry it. A template that needs a number of its own
  uses `og.ui.memory(id, initial)` / `og.ui.remember(id, value)`.
- Call UI only inside a system, and read and write the state you pass it
  through the archetype's column views (og-core §4;
  `templates/ui-kit/src/main.ts` shows the pattern).

## 2. Pick a template

Screenshots of every template at desktop (1280×720) and on a phone in portrait
(390×844 at 3×) are in the engine repository under `templates/ui-kit/screens/`;
the full-size reference frames are `crates/og-ui/references/metal/kit-*.png`,
and every style pack's screens on a phone `kit-skin-<pack>-phone.png`.

| The game needs | Call | It answers |
|---|---|---|
| A title screen | `og.ui.mainMenu(id, { title, subtitle, items, footer })` | `selected`: the item id |
| Pause | `og.ui.pauseMenu(id, { open, items })` (Resume, Settings, Quit by default) | `selected`; Back answers `resume` |
| Settings | `og.ui.settingsScreen(id, { open, settings, languages, onChange, onSave })` | the new `GameSettings`, `changed`, `closed` |
| A platformer HUD | `og.ui.platformerHud(id, { lives, coins, score, level, timeLeft, touch })` | `paused`, `moveX`, `action: "jump"` |
| A shooter HUD | `og.ui.shooterHud(id, { health, armor, ammo, reserve, abilities, map, touch })` | `paused`, `action` (ability id, `fire`), stick |
| A puzzle HUD | `og.ui.puzzleHud(id, { level, score, target, stars, moves, timeLeft })` | `paused` |
| An idle or incremental HUD | `og.ui.idleHud(id, { currencies, generators, tapLabel })` | `action: "tap"` or `"buy:<id>"` |
| A racing HUD | `og.ui.racingHud(id, { speed, gear, lap, laps, position, time, best, boost, map, touch })` | `paused`, steering, `held` |
| Dialogue or a cutscene | `og.ui.cutscene(id, { open, lines, index })` | the next `index`, `finished`, `skipped` |
| A shop | `og.ui.shopScreen(id, { balance, currency, items, categories })` | `bought`, `refused` (Buy pressed short of the price: it shook with the error sound), `closed` |
| An inventory | `og.ui.inventoryScreen(id, { slots, items })` | `selected`, `moved` (drag and drop), `used`, `closed` |
| Level select | `og.ui.levelSelect(id, { levels })` | `chosen`, `closed` |
| Results | `og.ui.resultsScreen(id, { open, score, best, newBest, stars, stats, shareText, onShare })` | `selected` (`retry`, `next`, `menu`, `share`) |
| A leaderboard | `og.ui.leaderboardScreen(id, { scopes, entries, loading, error })` | `scope`, `changed`, `retry`, `closed` |
| Onboarding | `og.ui.tutorialOverlay(id, { open, steps, index })` (each step names an element id) | the next `index`, `finished`, `skipped` |
| Loading | `og.ui.loadingScreen(id, { title, progress, tips })` | nothing |
| A yes/no question | `og.ui.confirmDialog(id, { open, title, message, confirmLabel, danger })` | `confirmed`, `cancelled` |
| A notice | `og.ui.alertDialog(id, { open, title, message })` | true when dismissed |
| A short message | `og.ui.toast(world, text, { tone })` (anywhere: systems, commands) | its id |

Compose them: a HUD with `pauseMenu` over it, `settingsScreen` opened from
pause, `confirmDialog` over settings. Dialogs stack and the topmost owns focus.

Honest data: `loadingScreen` shows a spinner when progress is unknown, never a
bar that runs ahead; `leaderboardScreen` shows skeletons while loading, the
error with Retry, and its empty text; never invent scores or counts.

## 3. Below the templates: the widget set

Layout (`row`, `column`, `panel`, `scroll`, `anchor`), text and controls
(`label`, `button`, `toggle`, `slider`, `textField`, `textArea`, `checkbox`,
`checkboxList`, `radio`, `tabs`, `select`, `stepper`, `chip`, `badge`,
`keyValue`, `divider`, `spacer`, `icon`, `image`, `progress`, `skeleton`,
`spinner`), HUD parts (`meter` for health, mana and xp with a trail that
follows a drop; `cooldown` rings; `minimap`; `dialogue` with typed text),
touch (`joystick`, `dpad`, `actionButton`, each at least 44 CSS px), lists
(`list` and `grid`, virtualised: only rows in view are built; items report
drag and drop; `scroll` with `axes: "horizontal"` or `"both"` for a strip
of cards or a map, which hands the wheel and drags it cannot use to the
scroller around it, keeps a flick's momentum and has grabbable
scrollbars; two fingers on a scroller scroll it on a phone, where one
finger would press a row), `zoomView` for a map or an artwork two fingers
pinch and pan (the wheel zooms it on a desktop), `aspect` on `image` (and
any element) to keep a picture's proportions in a growing column,
layers (`anchor` for HUD corners, `modal` for dialogs,
`transition` for fade, slide and scale by the tick, `spotlight`) and
`toast`. Ids are stable strings: give every control a meaningful id so a
playtest can `touch_move_to` it (`Shop/Panel/Detail/Buy` resolves as
`Detail/Buy`).

Mixed styles in one paragraph are `richText(id, markup)`: `[b]`,
`[color=danger]` (a token, or `#rrggbb`), `[size=lg]`, `[font=mono]`,
`[icon=coin]`, `[link=shop]…[/link]` and the per-glyph `[wave]`, `[shake]`,
`[pulse]`. Write the markup in the string table, not in code, so each
translation places its own emphasis (`confirmDialog` and `alertDialog`
messages are markup). A link is a child element (`Hint/shop`, kind `link`)
that keys, pads and touch reach like a button; the result's `link` names
the one clicked. A typo in a tag stays visible as text and warns.

`textField` and `textArea` edit as a player's system does, with nothing to
wire. On a Mac (the web shell says so: `Viewport.macKeys`) Cmd+A, C, X, Z
are the shortcuts, Cmd+←/→ go to the line's ends, Option+←/→ (and
Option+Backspace or Delete) move by words and Ctrl+A/E go to the line's
ends; elsewhere Ctrl takes the shortcuts and moves by words. Words are
Unicode's (UAX #29): `can't` is one, each Chinese character is one, Thai
moves a cluster at a time. A double click selects a word and a triple
click the line, Shift+Z or Y redoes (typing undoes a word at a time, an
input method's commit at a time), and a drag past an area's edge scrolls
it. A text the game sets starts the field's undo history again. Script a
chord with the modifier before the key: `key_down:Meta+ArrowLeft`
(Command), `key_down:Ctrl+Shift+KeyZ`; native hosts use the non-Mac keys,
so a script edits the same on every OS (`og.ui.previewViewport` with
`macKeys: true` tries the Mac's).

Scripts: a label mixing scripts draws each run in the first font that has
it (the Latin in the body or the pack's face, CJK, Arabic or Hebrew from the
locale's fonts or `fallbackFonts` in `og.ui.configure`), a letter and its
accents always in one face, so `"Level 3 · 古い水車小屋"` needs no font of
its own; a string a locale's font covers
whole keeps that face.

Icons: an icon set cooked from SVGs (an `.ogt` of `type icons` naming
them; the UI kit ships 64 iconoir icons as `icons/iconoir.ogt`) loads with
`og.load.font` and becomes the atlas with `og.ui.configure(world, { icons:
{ font } })`; `og.ui.icon(id, "heart", { size, tint })` and `[icon=heart]`
then draw crisp distance-field glyphs at any size in their ink. A texture
grid (`icons: { texture, columns, rows, names }`) still works for painted
icons. Icons sit in `fgMuted` or `fgSubtle` (design language: icons are
neutral). Sets name their icons differently (iconoir's `coins` and
`fire-flame`, the skin grids' `coin` and `fire`): an unknown name draws an
empty slot and warns, so a template that may meet either asks
`og.ui.hasIcon(name)` first and draws its fallback.

## 4. Focus: keyboard and gamepad

Every element carries an accessibility role, name, value and states
(`dump_ui`'s `role`, `name`, `value`, `states`; `og.ui.a11y()` answers the
tree with its focus order). Give every control visible text or a meaningful
id: an icon-only button is named by its icon, a card by the text inside it,
else by its id, and a playtest can require that no `focusable` element has
an empty name (ui-kit's `tests/a11y.og.ts`). Tab walks the tree's order.
On the web the shell mirrors this tree into the page for screen readers
(invisible ARIA elements over the canvas, toasts announced, the reader's
clicks pressing the control), so the names you give are what a blind
player hears. The page's `prefers-reduced-motion` and `prefers-contrast:
more` turn on the reduced-motion and high-contrast themes on top of the
game's own settings: nothing to wire. A stepper is adjusted as a whole
(VoiceOver's swipe up or down on it steps it), like a slider.

There are no label or role fields to set: a node's `role` comes from the
widget (button, link, text, heading, image, checkbox, slider, textbox,
progressbar, dialog, status…; layout containers have none) and its `name`
from its text, then the text inside it, then its id. So the work is in
the ids and texts: `"Buy"` reads well, `"btn3"` is what a blind player
hears. `og.ui.a11y()` answers `{ nodes, focusOrder, focused }`, each
`A11yNode` with `role`, `name`, `value`, `states` (disabled, focused,
checked, selected, expanded, busy…) and its rectangle. A screen reader's
own requests (focus, activate, increment, decrement) arrive as
`og.A11yAction` events and are applied to the control at the next frame
like a press. Prove it with a playtest that walks the tree:

```ts
const tree = og.ui.a11y();
const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "");
// a fact the playtest asserts is 0 (templates/ui-kit/tests/a11y.og.ts)
```

The mirror exists on the web only; natively the tree is still answered
for tests.

Arrows and the d-pad move focus to the nearest control that way (wrapping at
the edges), Tab and Shift+Tab walk reading order, Enter, Space and A press,
Escape and B go back: the topmost dialog or menu closes, otherwise
`og.ui.back()` answers true once (open pause on it). A gameplay HUD whose keys
move the player calls `og.ui.setNavigation(false)` (the HUD templates do);
dialogs turn navigation back on while open. The UI reads pad 0 from
`og.Gamepad` itself (d-pad, A, B, the left stick); for a custom binding,
queue intents with `og.ui.navigate(world, intent)`. A focus move plays the
`hover` sound and Enter, Space or A the `press` sound, as the pointer does
(og-audio §3). Script a pad in a
playtest with `gamepad:a` then `gamepad:-` (release), or `gamepad:-:0,0.9`
for the stick.

Local multiplayer on one screen (a lobby, a versus character select): call
`og.ui.setPlayers(4)` every tick the screen shows. Pads 1 to 3 then each
move a ring of their own (in the player's colour) among the same controls
and press with A; `response.pad` says whose press it was (null for the
pointer and keyboard, 0 for pad 0), `og.ui.focus(pad)` where each ring is.
Script them with `gamepad:right@2` then `gamepad:-@2`.

The touch controls (joystick, d-pad, action buttons) answer several fingers
at once: a thumb holding the stick and another on a button both count. Test
it with `touch_start` on the stick and `touch:1:down:<button id>` /
`touch:1:up:<button id>` for the second finger.

The UI is drawn over the finished frame (after bloom, grading and TAA), so
its colours are exactly the tokens and it stays sharp whatever camera the
game uses; a scroll list cuts its rows at the edge per pixel.

## 5. Phones, tablets and the safe area

Anchored groups sit inside the safe area (notch, home indicator) at
`topLeft` … `bottomRight` or `center`, inset by the density's gutter.
`og.ui.frameViewport().breakpoint` is `phonePortrait`, `phoneLandscape`,
`tablet` or `desktop`; templates rearrange themselves by it (tabs become a
select, side panels move under, touch controls appear when `touch` is set).
Preview a phone headless with a game command that calls
`og.ui.previewViewport(world, { width: 390, height: 844, dpr: 3, safeArea: { top: 47, bottom: 34, left: 0, right: 0 }, coarsePointer: true })`
and `og run --size 1170x2532`.

Gestures the UI owns: one finger presses and drags (a slider, a scroll's
flick, a drag and drop), two fingers on a scroller scroll it, two on a
`zoomView` pinch about their centre and pan it, and an `actionButton` is
pressed while held and clicked on a tap. There is no swipe, long-press or
double-tap event: a game that wants one reads `og.Pointers` (`down(id)`,
`start(id)`, `position(id)`; pointer 0 and up to four more) over ticks,
deciding by distance and ticks held, which replays exactly. A map or an
artwork to explore:

```ts
const z = og.ui.zoomView("Map", { texture: map, width: { kind: "grow" }, aspect: 16 / 9, maxZoom: 4 });   // ZoomResult: zoom, centerX, centerY (UI state)
```

A zoom view is a picture, not a control: keys do not move it. Script two
fingers with `touch_start` for the first and `touch:1:down:<x,y>` /
`touch:1:move:<x,y>` / `touch:1:up` for the second
(`templates/ui-kit/tests/gestures.og.ts`).

## 6. Theming: skins, motion, transitions, sounds

The default look is the platform's tokens; a game should have its own. A
**skin** (`og.load.skin`, an `.ogt` of `type skin`) replaces the palette,
radii, type ramp, durations, fonts and icons, and dresses every element
class in nine-slice frames from its atlas (panel, card, button with hover,
pressed and disabled, field, slot, tooltip, HUD capsule, progress track and
fill), with ornaments (panel corners, the divider, a heading flourish), a
panel texture, shadows and glows, press squash and hover lift, entrances and
sounds. Choosing one is og-taste §1. Six packs to start from are in
the engine repository under `templates/ui-kit/assets/skins/` (`pixel`,
`cartoon`, `fantasy`, `scifi`, `paper`, `neon`; copy the directory into the
game's `assets/skins/`):

```ts
const skin = og.load.skin(world, "skins/fantasy/skin.ogt");   // atlas, fonts, icons load with it
og.ui.configure(world, {
  skin: skin.ok ? skin.value : null,
  contrast: "high",                  // still applies over a skin
  reducedMotion: settings.reducedMotion,   // every motion becomes a fade
  sounds: { hover, press, open, close, success, error },  // og.load.sound handles, played on the ui bus
});
```

A skin's `slice` line says how a frame's edges and centre fill: `stretch`
(the default), `repeat` (tiled at the art's size from the corner, the last
tile cut) or `round` (a whole number of tiles resized to fit, as CSS
`round`): tile patterned borders (a rope, a rivet row, a pixel bevel),
stretch plain ones. `glass <part> <blur> <fill> [tint]` makes a part glass:
the game behind it blurred (`blur` CSS px), tinted by the part's fill, under
its frame drawn at `fill` opacity (the scifi pack's dialogs:
`glass panel 16 0.85 0.15`). Glass blurs the scene and the UI drawn under
it (a HUD under a glass dialog); it costs a frame copy and a blur pyramid
when any is on screen (about 0.8 ms at 1280×720 on an M4 Pro), and one more
pyramid for each glass layer with UI under it, nothing without glass.
A player whose system asks for less transparency (`prefers-reduced-
transparency`, or `reducedTransparency: true` in `og.ui.configure` from a
settings toggle) gets glass parts as opaque surfaces and dialog scrims and
translucent fills (secondary buttons, chips, tracks) flat. Your own
see-through overlays should follow: read `og.ui.frameTheme().reducedTransparency`
(and `reducedMotion`) and draw them opaque.

A skin of the game's own art: draw each part as a nine-patch PNG
(`panel.9.png`, `button.9.png`, `buttonHover.9.png`, ..., `soft.9.png`: the
art inside a one-pixel border whose top row and left column mark in opaque
black what stretches) and any ornament as a plain PNG (`corner.png`,
`divider.png`, `flourish.png`, `tile.png`) in the skin's directory, then
`og skin pack assets/skins/<name>`: it packs them into `atlas.png` and
writes the skin file's `atlas`, `slice` and `soft` lines and the ornaments'
regions, keeping every other line (palette, fonts, a slice's `tint` or
`repeat`). Re-run it after redrawing a part.

Never pass hex colours for chrome: use `{ token: "accent" }`, `"fgMuted"`,
`"surface"` and the type steps (`displayLg`, `xl`, `meta`); the skin's
palette answers them. A container that should look like a HUD capsule or a
card takes `frame: "capsule"` (or `"card"`, `"slot"`…) in its desc; titles
take `heading: true`. Without a skin both are plain.

Motion and juice, all by the tick and in the UI state:

| Want | Call |
|---|---|
| A score, coins or a reward counting up | `og.ui.counter(id, value, { from: 0, font: "display", step: "displayLg" })` |
| Sparks when a star or reward lands | `og.ui.burst({ count: 10 })` right after the element |
| A panel or item popping in | `og.ui.transition(id, show, { kind: "pop" })` (also `fade`, `slideUp`…, `scale`) |
| A change of game screen | `const s = og.ui.screenTransition("Screens", state.screen, { kind: "iris" }).shown;` then build `s` (`fade`, `wipe`, `iris`, `dissolve`, `slide`; a colour or a texture) |
| A wrong answer, a locked door, a refused purchase | `og.ui.shake({ sound: true })` right after the element, once, on the tick it happens: it swings three times and comes to rest by itself (the skin's `motion.shake`, else 6 px), with the `error` sound |

Skins add the rest themselves: dialog and panel entrances with staggered
controls, hover lift and press squash on springs (the `hoverSpring 1 350` and `pressSpring 0.55 120` lines of a
skin's `motion` block: damping ratio and response in ms; below 1 bounces), eased progress bars, toasts flying in,
a shake and the error sound on a press of a disabled button. Reduced motion
keeps every shake still and turns the rest into fades. How long and how
much is og-taste §5 and §6; camera shake is never for UI (og-feel §4).

## 7. Text: labels, fields, areas, input methods

Labels lay out by themselves: words break at spaces, CJK between
characters under kinsoku (og-i18n §3), right-to-left runs reorder, complex
scripts are shaped (Arabic joins, Devanagari conjuncts) and every font's
kerning pairs apply. What a game sets:

```ts
og.ui.label("Quest", quest.text, { width: { kind: "grow" }, maxLines: 2, ellipsis: true, color: { token: "fgMuted" } });   // cut with … on the last line
og.ui.label("Gold", gold, { font: "rounded", step: "xl", nowrap: true });   // body, display, mono or rounded
```

`ellipsis` cuts a `nowrap` label (which may then shrink in its row) or the
last of `maxLines`; the probe still reads the whole text, so a playtest
can find it. Keep long text in a `scroll` instead of cutting what the
player must read.

Text entry is a field (one line) or an area (several), each one call that
takes the value and answers it, like every widget (§1):

```ts
// Save = defineResource("Save", { name: str(64), note: str(1024) }): maxChars × 4 bytes always fits.
const name = og.ui.textField("Name", save.name, { placeholder: "Your name", maxChars: 16 });
if (name.response.changed) save.name = name.value;
if (name.response.submitted) state.screen = "play";   // Enter, once
const note = og.ui.textArea("Note", save.note, { placeholder: "A note for later", rows: 3, maxChars: 256, width: { kind: "grow" } });
save.note = note.value;
```

Both take the press point as the caret, select by drag, Shift with
arrows, and Cmd (on a Mac) or Ctrl with A; copy, cut and paste use the page's
clipboard; typed text replaces the selection; the text scrolls to keep the
caret in view and the caret blinks by the tick (steady under reduced
motion). An area's Enter is a new line, Up and Down move between lines
and stay out of focus navigation while it is edited, and `maxChars`
counts its line breaks. On the web the shell keeps a hidden text box over
the focused field, so a phone's keyboard opens and an input method
(Chinese, Japanese, Korean) shows its candidates there; the composition
is drawn underlined at the caret until it commits. Script them:
`type:<text>`, `compose:<text>` (`compose:` alone ends it) and
`commit:<text>`, then read the field's value from `dump_ui` or the
game's facts (`templates/ui-kit/tests/text.og.ts`).

Fonts: a label takes the role's font when it can draw the whole text,
else the locale's fonts, then `UiDesc.fallbackFonts` (a Hebrew or
Devanagari face, a CJK face cooked with `runtime true`), then the skin's
`fallback`. One label uses one font: a Hebrew word next to digits needs a
Hebrew font that has digits. Tracking applies to Latin text only.

## 8. Proving it

`templates/ui-kit` in the engine repository runs every template; its
playtests show how to drive UI from a script: `move_to_text:=shop` then
`touch_start`/`touch_end`, `key_down:Tab`/`key_up:Tab`, a game command that
calls `og.ui.navigate`, `dump_ui` for bounds and texts, and frame comparison
against references. `og polish .` scores the screens' contrast, text size,
transitions and sounds (og-build §5).

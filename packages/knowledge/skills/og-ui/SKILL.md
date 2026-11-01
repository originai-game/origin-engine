---
name: og-ui
description: Game screens and HUDs on Origin Engine with og.ui — which screen template to pick (title, pause, settings, five HUDs, cutscene, shop, inventory, level select, results, leaderboard, tutorial, loading, dialogs, toasts), how to feed it data and read what it answers, the widget set beneath, focus for keyboard and gamepad, accessibility (the screen-reader mirror), phones, touch gestures and the safe area, text (fields, text areas, selection and the clipboard, input methods, ellipsis, kerning, shaping, font fallback), and the look: UI styles (a look's `ui` line, the six built-ins, the game's own) and how their frames tile, count-ups, bursts, shakes, screen transitions and UI sounds, and proving a UI with og ui preview and og ui check. Use when building or changing menus, HUDs or any UI in a project with a project.ogt.
---

`og.ui` is the engine's immediate-mode UI: a system describes the tree every
tick, the engine lays it out, draws it in the game's UI style (a look's
`ui` line; the `prototype` placeholder without one, never the platform's
tokens) and answers the probe (`dump_ui`, `touch_move_to`, `move_to_text`). The API
reference is `og api show ui`; this skill is how to use it well.

## What works

Two games written from this package alone (`examples/lantern-vale`,
`examples/hollowbell`) used the main menu, pause, cutscene, shop and
results templates (the results with a localised Best and New best) and a
text field with input-method composition, and none needed wiring beyond
its desc; a style copied into the game restyled every screen, and
`og polish` measured taste 99 for both. The Kestrel Stair
(`examples/kestrel-stair`) used the cutscene, tutorial overlay,
inventory with drag and drop, level select, leaderboard with its honest
states, settings, results and pause: each worked from its desc, the paper pack
restyled them all, and every word they draw came from its string tables.

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
and every style pack's screens on a phone `kit-style-<pack>-phone.png`.

| The game needs | Call | It answers |
|---|---|---|
| A title screen | `og.ui.mainMenu(id, { title, subtitle, items, footer, instant })` (`instant` while it is the screen the game opens on: §6) | `selected`: the item id |
| Pause | `og.ui.pauseMenu(id, { open, items })` (Resume, Settings, Quit by default) | `selected`; Back answers `resume` |
| Settings | `og.ui.settingsScreen(id, { open, settings, languages, world, onChange, onSave })` (with `world`, a new graphics preset is applied: `og.render.applyQuality`) | the new `GameSettings`, `changed`, `closed` |
| A platformer HUD | `og.ui.platformerHud(id, { lives, maxLives, coins, score, level, timeLeft, touch })` (what it is not given it does not draw: `maxLives: 0` no hearts, no `coins` or `score` no box for them; the words `pauseLabel`, `jumpLabel` from the string table, og-i18n §2) | `paused`, `moveX`, `action: "jump"` |
| A shooter HUD | `og.ui.shooterHud(id, { health, armor, ammo, reserve, abilities, map, touch })` (the words `killsLabel`, `fireLabel`, `pauseLabel`) | `paused`, `action` (ability id, `fire`), stick |
| A puzzle HUD | `og.ui.puzzleHud(id, { level, score, target, stars, moves, timeLeft })` (the words `movesLabel`, `targetLabel` with `{target}`, `pauseLabel`) | `paused` |
| An idle or incremental HUD | `og.ui.idleHud(id, { currencies, generators })` (the words `tapLabel`, `levelLabel` with `{level}`, `rateLabel` with `{rate}`, `pauseLabel`) | `action: "tap"` or `"buy:<id>"` |
| A racing HUD | `og.ui.racingHud(id, { speed, gear, lap, laps, position, racers, time, delta, best, splits, boost, map, touch })` (a time trial: `racers: 0` hides the place, `boostMeter: false` the meter, `delta` and `splits: [{ label, time, delta }]` show its sectors; the words `lapLabel`, `bestLabel`, `pauseLabel`, `boostLabel`, `brakeLabel`, `leftLabel`, `rightLabel`, `unit` from the string table, og-i18n §2) | `paused`, steering, `held` |
| Dialogue or a cutscene | `og.ui.cutscene(id, { open, lines, index })` | the next `index`, `finished`, `skipped` |
| A shop | `og.ui.shopScreen(id, { balance, currency, items, categories })` | `bought`, `refused` (Buy pressed short of the price: it shook with the error sound), `closed` |
| An inventory | `og.ui.inventoryScreen(id, { slots, items })` | `selected`, `moved` (drag and drop), `used`, `closed` |
| Level select | `og.ui.levelSelect(id, { levels, backLabel, starsLabel, lockedLabel })` (a card is named for a screen reader by its level and `starsLabel`, "2 of 3 stars", a locked one by `lockedLabel`, "3, locked") | `chosen`; `closed` from its Back button, a tap on the scrim, Escape or B |
| Results | `og.ui.resultsScreen(id, { open, score, best, newBest, stars, stats, shareText, onShare, scoreLabel, bestLabel, newBestLabel })` (the labels from the string table, og-i18n §2); a time trial passes `format: "time"` with the score and best in seconds: the headline counts up as m:ss.mmm (1:36.233) and the lower time is the best | `selected` (`retry`, `next`, `menu`, `share`) |
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

### Components: containers, navigation and data

Between the templates and the raw widgets sit the components (one call
per tick, data in, what happened out; keep what they answer and pass it
back). Each takes pointer, touch, keys and pads, mirrors under a
right-to-left locale, says its role to a screen reader and plays the
style's sounds; a style dresses its parts (`Menu.item[hover]`,
`Tabs.indicator`, `Card.frame[selected]`, `Chart.tooltip`).

| The game needs | Call | It answers |
|---|---|---|
| A card (an item, a quest, a mode) | `og.ui.card(id, { title, subtitle, eyebrow, media, badge, interactive, selected }, build)` | its response (`clicked`) |
| Content that scrolls | `og.ui.scrollArea(id, { height, axes }, build)` (edges fade where more lies) | nothing |
| FAQ or grouped settings | `og.ui.accordion(id, { sections, expanded, single }, build(i))` | `expanded`, `toggled` |
| A list beside its detail | `og.ui.splitView(id, { ratio, direction }, start, end)` (stacks on a phone) | `ratio` |
| A dialog with actions | `og.ui.dialog(id, open, { title, message, tone, actions }, build)` | `action`, `dismissed` |
| A yes/no in a component | `og.ui.confirm(id, open, { title, message, confirmLabel, danger })` | `confirmed`, `cancelled` |
| Details beside an element | `og.ui.popover(id, open, { anchor: og.ui.lastPath(), side }, build)` | `dismissed` |
| Commands, a context menu | `og.ui.menu(id, open, { anchor or at, entries })`, `og.ui.contextMenu(id, open, at, { entries })` (check, radio, separator, header, danger, submenus by `parent`) | `chosen`, `dismissed` |
| A bottom or side sheet | `og.ui.sheet(id, open, { edge, detents, detent, title }, build)` | `detent`, `dismissed` |
| Tabs | `og.ui.tabBar(id, { tabs, selected, variant, bumpers }, build(i))` (Q/E, LB/RB step) | `selected` |
| A banner strip | `og.ui.carousel(id, { count, index, autoplayMs }, build(i))` | `index` |
| Where the player is | `og.ui.breadcrumbs(id, { items })` | `chosen` |
| Pages of results | `og.ui.pagination(id, { pages, page })` | `page` |
| The action bar | `og.ui.actionHints(id, { hints: og.ui.actionBar(world) })` | `pressed` (a touch on a hint) |
| A weapon or emote wheel | `og.ui.radialMenu(id, open, { items, aim, release })` | `chosen`, `hovered` |
| Long lists and grids with states | `og.ui.dataList(id, { count, loading, error, emptyLabel }, build(i))`, `og.ui.dataGrid(...)` | `clicked`, `retry` |
| A leaderboard or stats table | `og.ui.table(id, { rows, columns, sortBy, descending, filter })` (10,000 rows and more: only the rows in view are built) | `sortBy`, `descending`, `filter`, `clicked` |
| A codex | `og.ui.tree(id, { items, expanded, selected })` | `chosen`, `expanded` |
| A skill tree, a run map | `og.ui.nodeGraph(id, { nodes, links, zoom })` | `chosen`, `zoom` |
| A chart | `og.ui.chart(id, { kind: "line" \| "area" \| "bar" \| "radar" \| "pie" \| "donut", categories, series })` | `hovered`, `clicked` |
| A stat sheet, an equip comparison | `og.ui.keyValueRows(id, { rows })`, `og.ui.statCompare(id, { rows: [{ label, current, next, max }] })` | nothing |
| Daily rewards, events | `og.ui.calendar(id, { year, month, today, selected, marks })` (pass today in: no clock in the tick) | `selected`, `month`, `year` |

`templates/ui-kit` shows each on its `containers`, `navigation`,
`overlays`, `data`, `charts` and `graph` screens (`demo:overlay:<name>`,
`demo:data:<loading|empty|error>` pick a state).

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

**Text over a world point** (a name tag over a critter, a marker over a
door): project the point each tick with the og.Camera resource's
`worldToScreen` (null behind the camera: draw nothing) and draw screen
text there, not world text. World text (`og.render.text` without `screen`) faces +z, rides
the tone curve and is blurred by depth of field at the frame's edge;
screen text is never tone mapped, blurred or depth tested, so the tag
stays crisp and in its colour, and at a `layer` up to `UI_LAYER` (1000) it
draws under every UI element (the HUD and the dialogs cover it):

```ts
const Tags = defineSystem("Tags", { reads: [Game, og.Camera] }, (_q, _t, world) => {
  const cam = must(world.resource(og.Camera));
  for (const c of critters(world)) {
    const at = cam.worldToScreen({ x: c.x, y: c.y + 1.25, z: c.z }); // CSS pixels, y down
    if (at === null) continue;
    og.render.text(world, { font, text: c.name, position: { x: at.x, y: at.y, z: 0 }, size: 18, align: "center", outline: 0.7, screen: true, layer: 10 });
  }
});
```

Keep the font in a `Font()` field. Two tags within a tag's width of each
other overlap into one label: lift the later one by its height, or hide
the farther one's.

Mixed styles in one paragraph are `richText(id, markup)`: `[b]`,
`[color=danger]` (a token, or `#rrggbb`), `[size=lg]`, `[font=mono]`,
`[icon=coin]`, `[link=shop]…[/link]` and the per-glyph `[wave]`, `[shake]`,
`[pulse]`. Write the markup in the string table, not in code, so each
translation places its own emphasis (`confirmDialog` and `alertDialog`
messages are markup). A link is a child element (`Hint/shop`, kind `link`)
that keys, pads and touch reach like a button; the result's `link` names
the one clicked. A typo in a tag stays visible as text and warns.

`textField` and `textArea` edit as a player's system does, with nothing to
wire. On a Mac (`Viewport.macKeys`: the web shell sets it from the
browser's platform, a native host on macOS) Cmd+A, C, X, Z are the
shortcuts, Cmd+←/→ go to the line's ends, Option+←/→ (and
Option+Backspace or Delete) move by words, and Ctrl takes the emacs keys
(A/E the line's ends, F/B a character, N/P a line, D/H delete one, K
kills to the paragraph's end, U to its start, W the word before, Y yanks
what they killed, T transposes); elsewhere Ctrl
takes the shortcuts and moves by words. Words are Unicode's (UAX #29):
`can't` is one, each Chinese character is one, Thai moves a cluster at a
time. The caret and Backspace step over a whole grapheme cluster (`é`
typed as e and an accent, an emoji with its skin tone), and ←/→ move on
screen: in Hebrew or Arabic ← goes on in the text, and across the edge of
a Hebrew word in a Latin line the caret goes where the arrow points. A double click selects a word and a triple
click the line, Shift+Z or Y redoes (typing undoes a word at a time, an
input method's commit at a time), and a drag past an area's edge scrolls
it. A text the game sets starts the field's undo history again. Script a
chord with the modifier before the key: `key_down:Meta+ArrowLeft`
(Command), `key_down:Ctrl+Shift+KeyZ`. A native run on macOS takes the
Mac's keys, so a script that holds Ctrl or Meta and must edit the same on
every OS pins them with `og.ui.previewViewport` (`macKeys`).

Scripts: a label mixing scripts (`"Level 3 · 古い水車小屋"`) needs no font
of its own; which face draws which part is one rule, §7 (Fonts).

Icons: an icon set cooked from SVGs (an `.ogt` of `type icons` naming
them; the UI kit ships 72 iconoir icons as `icons/iconoir.ogt`, also under
its own names `coin`, `gear`, `fire`…, and every kit screen draws them) loads with
`og.load.font` and becomes the atlas with `og.ui.configure(world, { icons:
{ font } })`; `og.ui.icon(id, "heart", { size, tint })` and `[icon=heart]`
then draw crisp distance-field glyphs at any size in their ink. A texture
grid (`icons: { texture, columns, rows, names }`) still works for painted
icons. Icons sit in `inkMuted` or `inkFaint` (icons stay neutral; colour
is for state). Sets name their icons differently (iconoir's `coins` and
`fire-flame`, the style grids' `coin` and `fire`): an unknown name draws an
empty slot and warns, so a template that may meet either asks
`og.ui.hasIcon(name)` first and draws its fallback, or names an icon twice
in its set (`icon "coin" "coins"`). A path's `stroke-linejoin` is kept
(a miter stays sharp); a set whose SVGs leave it out says `join round` (or
`miter`, `bevel`) once.

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
(VoiceOver's swipe up or down on it steps it), like a slider, and both
tell the reader their bounds (a node's `valueMin`, `valueMax`, `valueNow`:
a slider's `min` and `max`, a stepper's when it has them).

There are no role fields to set: a node's `role` comes from the
widget (button, link, text, heading, image, checkbox, slider, textbox,
progressbar, dialog, status…; layout containers have none) and its `name`
from its text, then the text inside it, then the label just before it in
its row, then its id. A group a player never reads as text names itself
from its desc's `label` (`tabs`, `radio`, `select`: "Apartado", not the id
`Section`), and an icon or image from its `label`: an icon left without one
is named by its atlas name, an English word (`star`), so a card of a name
and three stars reads "1. La escalera star star star" in every language.
Name the meaningful one in the game's words and leave the rest `""`
(decoration): `og.ui.icon("Star#0", "star", { label: og.i18n.t(world,
"stars", { n: 2 }) })`, then `{ label: "" }` for the others. A text field is named by that label (put the label
and the field in one `row`), else by its placeholder, and the placeholder
is its `description`: `"Lamplighter"`, then `"Your name"`, never the text
typed in it (that is its `value`). So the work is in the ids and texts:
`"Buy"` reads well, `"btn3"` is what a blind player hears. `og.ui.a11y()` answers `{ nodes, focusOrder, focused }`, each
`A11yNode` with `role`, `name`, `description`, `value`, `states` (disabled, focused,
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
`og.ui.back()` answers true once (open pause on it). While a dialog or menu
is open `og.ui.back()` answers false even when asked before the dialog is
built that tick, so a HUD drawn under pause or the settings never takes the
Back that closes them (a locked dialog, which Back does not close, lets it
through). A gameplay HUD whose keys
move the player calls `og.ui.setNavigation(false)` (the HUD templates do);
dialogs turn navigation back on while open. What opens takes the focus:
a dialog (every modal template: pause, settings, shop, results, confirm)
its first control, a dialogue box or cutscene the box, without a Tab first.
An entering panel is inert until its entrance finishes: Enter, Space, A
and pointer presses during that motion are ignored, not buffered. Its
controls answer once the panel arrives, even if their stagger continues.
Use `instant: true` on a `mainMenu` or `anchor` to skip its entrance when
immediate input is needed (for example under a screen wipe, §6).
The UI reads pad 0 from
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
They also keep out of the host's own control over the canvas: on a
published page the web shell's round Back to the Portal sits top left
(about 12, 12 to 52, 52 CSS px) and says where in
`og.ui.frameViewport().covered`, so a group it would overlap moves along
its edge past it (the racing HUD's place and lap to its right), or, in a
corner where that would land it on another group (a shooter's vitals on a
phone), down past it. Every HUD template and any HUD built from
`og.ui.anchor` groups gets this; a corner drawn another way (screen
sprites, a row placed by hand) reads `covered` itself, so put a game's own
HUD in anchored groups. Preview it
with `og.ui.previewViewport(world, { ..., covered: { x: 12, y: 12, width: 40, height: 40 } })`,
or give a native run the page's cover from outside the game with the
command `cover:12,12,40,40` (`og run --setup cover:12,12,40,40`; a
playtest's `setup: "cover:12,12,40,40"`, og-build §6), `cover:none`
clearing it.
On a wide PC screen (21:9 up to 32:9) anchored groups and toasts keep to a
centred band no wider than `UiDesc.hudMaxAspect` (21/9 by default;
`og.ui.configure(world, { hudMaxAspect: 16 / 9 })` for a tighter one, 0
for the whole screen), so the vitals stay in the player's view; modals and
world-space UI keep the whole screen, and nothing stretches. The band is
`og.ui.viewport().hud`; a HUD placed by hand reads it. The settings
template's Graphics tab is the PC graphics page when
`GameSettings.display` holds `og.render.display(world).settings`
(preset, render scale, frame-rate limit, V-Sync, adaptive quality,
battery saver, fullscreen, GPU with "Applies after a restart"), and its
Controls tab sets the mouse's sensitivity on `SettingsDesc.lookAction`
and Raw mouse input (`GameSettings.rawMouse`).
`og.ui.frameViewport().breakpoint` is `phonePortrait`, `phoneLandscape`,
`tablet` or `desktop`; templates rearrange themselves by it (tabs become a
select, side panels move under, touch controls appear when `touch` is set).
Preview a phone headless with a game command that calls
`og.ui.previewViewport(world, { width: 390, height: 844, dpr: 3, safeArea: { top: 47, bottom: 34, left: 0, right: 0 }, coarsePointer: true })`
and `og run --size 1170x2532`.

Gestures the UI owns: one finger presses and drags (a slider, a scroll's
flick, a drag and drop), two fingers on a scroller scroll it (and fling it
as they lift), two on a
`zoomView` pinch about their centre and pan it, and an `actionButton` is
pressed while held and clicked on a tap. There is no swipe, long-press or
double-tap event: a game that wants one reads `og.Pointers` (`down(id)`,
`start(id)`, `position(id)`; pointer 0 and up to four more) over ticks,
deciding by distance and ticks held, which replays exactly. A map or an
artwork to explore:

```ts
const z = og.ui.zoomView("Map", { texture: map, width: { kind: "grow" }, aspect: 16 / 9, maxZoom: 4 });   // ZoomResult: zoom, centerX, centerY (UI state)
```

Tab (or a pad) reaches a zoom view: `+` and `-` zoom it, `0` fits it
again, the arrows pan it (they stay its own while it is focused). Script two
fingers with `touch_start` for the first and `touch:1:down:<x,y>` /
`touch:1:move:<x,y>` / `touch:1:up` for the second
(`templates/ui-kit/tests/gestures.og.ts`).

## 6. Styles: the look of the game's UI, motion, transitions, sounds

The game's UI follows the game's art, never the Origin platform's look:
the engine has no house style to fall back on. Without a style a UI draws
in **prototype**, the engine's placeholder: greys, near-square corners and
a cyan focus ring, meant to be replaced. `og polish` warns when a game
ships in it. A **style** (`og.load.style`, an `.ogt` of `type style`,
`version 2`) is a whole design system: its tokens (palette roles, semantic
colours, typography by role and breakpoint, space and size scales, radii,
strokes, elevation, a shape language per element class, motion, sounds and
haptics) and its **dressings**, which draw every part of every widget with
paints, strokes, effects and materials (or nine-slices from its atlas). A
part a style does not dress draws from its tokens and still looks
finished. Choosing one is og-taste §1.

The grammar, block by block (`templates/styles/minimal/style.ogt` is a
short complete one; `og style upgrade <file>` writes a version 1 style as
version 2, keeping how it draws):

```text
type style
version 2
---
name "arena"
scheme dark
# Every palette role (backdrop, panel, ink, inkMuted, primary, focus, ...).
palette {
  backdrop "#0a1431"
  panel "#1c2540"
  ink "#f5f7ff"
  primary "#e8322b"
}
# Semantic colours: status icons and patterns, rarities with pips, teams, elements, chart series.
colors {
  success icon "check"
  rare "#2f7fef" pips 3
  team1 "#3d7eff"
}
# A role: size, line height, tracking (em; past 0.05 it is set in capitals), weight, font; per breakpoint.
typography {
  display 48 54 -0.01 700 display {
    phonePortrait 38 43
    phoneLandscape 43 48
  }
  body 15 22 0 400 body
  label 12 17 0.08 600 body
  numeric 19 26 0 500 numeric
}
space {
  sm 8
  md 12
}
size {
  touch 44
}
stroke {
  hairline 1
  focus 2
}
elevation {
  raised {
    shadow 0 2 6 0 "#00000040"
  }
}
# Each element class: round, squircle, bevel, notch or square, then a radius or pill.
shape {
  panel bevel 10
  button round pill
  slot notch 6
}
fonts {
  body "Rajdhani-Bold.ttf"
  display "@styles/neon/ChakraPetch-BoldItalic.ttf"
  cjk "@styles/cjk/NotoSansSC-Bold.ttf"
}
motion {
  spring ui 150 0
  spring playful 400 0.18
  ease standard 0.2 0 0 1
  duration fast 140
  stagger 30
  overshoot 0.06
}
# Every event of docs/ui.md §7.8 (hover to revealMythic): a set, picked never twice running, its ranges, a haptic.
sounds {
  press "tap.ogt" "tap2.ogt" volume 0.5 0.65 pitch 0.96 1.04 haptic tick
  error "error.ogt" volume 0.6 0.7 pitch 1 1.02 haptic error
  hover none haptic none
}
dress "Button.base" {
  fill linear 180 controlStrong control
  stroke 2 lineStrong inside
  effect dropShadow 0 4 0 0 "#0000006c"
  press 1 0.96 2
  hover 0 1
}
dress "Button.base[primary]" {
  fill linear 180 primary primaryActive
}
dress "Button.base[primary][pressed]" {
  effect innerShadow 0 3 0 0 "#0000002e"
}
dress "Slot.frame[rarity=epic]" {
  stroke 2 "#a855f7" inside
  effect outerGlow 8 "#a855f766"
}
```

A selector is `Component.part`, then any of `[variant]`, `[state]`
(`hover`, `pressed`, `focused`, `disabled`, `selected`, `on`, `low`) and
`[attr=value]`; the more specific wins and lines of the same selector merge.
The parts: `Panel.frame`, `Card.frame`, `Dialog.frame`, `Tooltip.frame`,
`Hud.frame`, `Button.base`, `Field.frame`, `Slot.frame`, `Meter.track`,
`Meter.fill[kind=health]`, `Tabs.track`, `Tabs.tab`, `Chip.base`,
`Badge.base[tone=danger]`, `Checkbox.box`, `Radio.box`, `Toggle.track`,
`Slider.track`; an element names its own part, with attributes the game
chooses, in its desc (`{ part: "ShopCard.frame[rarity=epic]", partClass:
"card" }`). Name colours by palette role (`primary`, `lineStrong`) so a
game's `primaryColor` and high contrast follow; a quoted `"#rrggbbaa"` is
fixed. A `fill vector "<asset>.svg"` paints vector art the style's asset
binding loads. Every state moves on the style's `spring ui`, so hover,
press and selection never snap.

Haptics: each event's pattern reaches the pad's rumble and the phone's
vibration on both hosts; the player's setting scales or silences them
(`og.ui.setHaptics(world, 0..1)`, 0 off, the settings screen's vibration
switch). `og.ui.feedback(world, "reward")` plays an event's sound and
haptic by hand.

Art slots (docs/ui.md §13): a UI document's `art` element is a slot (its
kind, aspect, focal point); it draws the slot's texture, or until it lands
the style's placeholder for its kind (a gradient of the style's colours
and the kind's icon, never a grey box). `og ui slots [project] [--json]` lists every slot of a game with
its spec (kind, aspect, focal point, safe area, fit) and fails on a bad one.

**A new style for the game's look**: `og style new <name> --look "<family>"
--premise "<one line>"`, then any of `--seed "#rrggbb"`, `--scheme dark|light`,
`--shape soft|round|pill|squircle|sharp|cut|notch|pixel`,
`--ornament none|light|rich`,
`--motion calm|snappy|bouncy|heavy|mechanical|floaty` and
`--finish flat|gradient|glass|hologram|neon|paper|bevel|brush`. It starts
from the seed style nearest the look, writes `assets/styles/<name>/` (the
style with every ink held to its contrast bar, its `type sfx` sounds, and
`art-requests.json`: the frames, buttons, ornaments and icons it asks for,
which the art brief's `styleArtBriefs` turns into prompts) and prints the
`ui` line. It draws finished before any art lands; read its contact sheet
(§8), then tune its values.

The usual way in is the game's look (og-taste §12): its `ui` line names
the style, and every catalogue preset already names one. `ui "pixel"` is a
built-in style by slug (`og style list`: the atlas packs `pixel`, `cartoon`,
`fantasy`, `scifi`, `paper`, `neon`, and the generated seeds `gilded`,
`storybook`, `tactical`, `holo`, `cyber`, `bubbly`, `kawaii`, `arcade`,
`craft`, `minimal`, `dread`, `broadcast`, `inkwash`, one per look family;
`og cook` bundles the one a look names and the fonts and icons it shares). `ui
"styles/<name>/style.ogt"` is the game's own, and a game's
`assets/styles/<slug>/style.ogt` wins over the built-in of that slug.
To make one from a built-in, run `og style eject <slug> <name>` (`og style
list` names the slugs): it writes the built-in's style.ogt, atlas, icons,
fonts with their import settings and licences into `assets/styles/<name>/`
(never over existing files without `--force`) and prints the `ui
"styles/<name>/style.ogt"` line for the look. Then edit palette roles,
tokens and dressings in `assets/styles/<name>/style.ogt`; a style's ids are
read relative to its file first, then from `assets/`, and its `cjk` fonts
stay the shared built-in `@styles/cjk/` ones. A style file must name every
palette role; `og cook` lists what is missing or malformed. Loading
one by hand:

```ts
const style = og.load.style(world, "styles/fantasy/style.ogt");   // atlas, fonts, icons load with it
og.ui.configure(world, {
  style: style.ok ? style.value : null,
  contrast: "high",                  // still applies over a style
  reducedMotion: settings.reducedMotion,   // every motion becomes a fade
  sounds: { hover, press, open, close, success, error },  // og.load.sound handles, played on the ui bus
});
og.ui.configure(world, { reducedMotion: true });   // later, from the settings: the style, fonts and sounds stay
og.ui.configure(world, { primaryColor: "#e0553b" });   // re-colour the style's primary; hover, pressed and the ink on it follow
```

`configure` takes a partial UiDesc: a field left out keeps the value the
last call gave it (or the style a look applied, `GameDesc.look` and
`og.render.applyLook`), and `null` clears one (`style: null` is the
prototype style again). A UI left with no font at all warns that it draws
no text.

A style's `slice` line says how a frame's edges and centre fill: `stretch`
(the default), `repeat` (tiled at the art's size from the corner, the last
tile cut) or `round` (a whole number of tiles resized to fit, as CSS
`round`): tile patterned borders (a rope, a rivet row, a pixel bevel),
stretch plain ones. `glass <part> <blur> <fill> [tint]` makes a part glass:
the game behind it blurred (`blur` CSS px), tinted by the part's fill, under
its frame drawn at `fill` opacity (the scifi style's dialogs:
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

A style of the game's own art: draw each part as a nine-patch PNG
(`panel.9.png`, `button.9.png`, `buttonHover.9.png`, ..., `soft.9.png`: the
art inside a one-pixel border whose top row and left column mark in opaque
black what stretches) and any ornament as a plain PNG (`corner.png`,
`divider.png`, `flourish.png`, `tile.png`) in the style's directory, then
`og style pack assets/styles/<name>`: it packs them into `atlas.png` and
writes the style file's `atlas`, `slice` and `soft` lines and the ornaments'
regions, keeping every other line (palette, radii, fonts, a slice's `tint` or
`repeat`). Re-run it after redrawing a part.

Never pass hex colours for chrome: use `{ token: "primary" }`, `"inkMuted"`,
`"panel"` and the type steps (`xl3`, `xl`, `meta`); the style's
palette answers them. A container that should look like a HUD box or a
card takes `frame: "hud"` (or `"card"`, `"slot"`…) in its desc; titles
take `heading: true`. Without a frame atlas both are plain fills at the
style's radii.

Motion and juice, all by the tick and in the UI state:

| Want | Call |
|---|---|
| A score, coins or a reward counting up | `og.ui.counter(id, value, { from: 0, font: "display", step: "xl3" })` |
| Sparks when a star or reward lands | `og.ui.burst({ count: 10 })` right after the element |
| A panel or item popping in | `og.ui.transition(id, show, { kind: "pop" })` (also `fade`, `slideUp`…, `scale`) |
| A change of game screen | `const s = og.ui.screenTransition("Screens", state.screen, { kind: "iris" }).shown;` then build `s` (`fade`, `wipe`, `iris`, `dissolve`, `slide`; a colour or a texture) |
| A wrong answer, a locked door, a refused purchase | `og.ui.shake({ sound: true })` right after the element, once, on the tick it happens: it swings three times and comes to rest by itself (the style's `motion.shake`, else 6 px), with the `error` sound |

`screenTransition` keeps the previous screen in `shown` for the closing
half of its duration, switches at full cover, then reveals the new one.
Build `shown`, not the requested screen; `dump_ui` therefore lists the old
screen until the midpoint. The cover itself does not block input. A new
panel's own style entrance starts when that panel is first built, and
ignores presses until it arrives (§4): the closing half of a wipe plus
that entrance explains why a press just after the switch can be lost.

Styles add the rest themselves: dialog and panel entrances with staggered
controls, hover lift and press squash on springs (the `hoverSpring 1 350` and `pressSpring 0.55 120` lines of a
style's `motion` block: damping ratio and response in ms; below 1 bounces), eased progress bars, toasts flying in,
a shake and the error sound on a press of a disabled button. Reduced motion
keeps every shake still and turns the rest into fades. The screen the game
opens on is built settled (`instant: true` on its `mainMenu`, `anchor` or
`transition` while it is that screen): the web draws it behind the play
gate, where an entrance would stand part-way until the tap (og-platform-sdk
§3). How long and how
much is og-taste §5 and §6; camera shake is never for UI (og-feel §4).

## 7. Text: labels, fields, areas, input methods

Labels lay out by themselves: words break at spaces, CJK between
characters under kinsoku (og-i18n §3), right-to-left runs reorder, complex
scripts are shaped (Arabic joins, Devanagari conjuncts) and every font's
kerning pairs apply. What a game sets:

```ts
og.ui.label("Quest", quest.text, { width: { kind: "grow" }, maxLines: 2, ellipsis: true, color: { token: "inkMuted" } });   // cut with … on the last line
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
`type:<text>` (the text to the `;` exactly, a trailing space included),
`compose:<text>` (`compose:` alone ends it) and `commit:<text>`, then read
the field's value from `dump_ui` or the game's facts
(`templates/ui-kit/tests/text.og.ts`). Fields take the typed-text stream
(`og.TextInput`: the page's typing, `type:` and `commit:`); until a world
has had any, a printable key pressed (`key_down:KeyM`) types its US-layout
character, and from the first on keys only edit (Backspace, arrows), so
script text with `type:`, not `key_down`.

Fonts (the one rule for every label, field and button):

1. The role's font (body, display, mono, rounded; the style's or
   `og.ui.configure`'s) when it has every character: the whole label in it.
   A style with Chinese companions (`fonts { cjk cjkDisplay }`; every
   built-in style has them, matched to its Latin voice: Noto Serif SC for
   fantasy, ZCOOL KuaiLe for cartoon, Fusion Pixel for pixel, Noto Sans SC
   Bold for neon) sets a label's Chinese in its companion and its Latin in
   the role's face, ahead of steps 2 and 3, so a Chinese title is not
   Noto Sans Regular beside a display face. The cook brings the companions
   only into a game with a `zh` string table. A game's own style names
   them as `cjk "@styles/cjk/NotoSerifSC-SemiBold.ttf"` (the faces:
   templates/styles/cjk/LICENSES.md).
2. Else the first of the locale's fonts (its string table's `fonts` line)
   that has every character: the whole label in that face, its Latin and
   digits too. A CJK subset that carries Latin-1, as the archetypes cook
   theirs, therefore sets `"Lantern Vale · 灯笼谷"` entirely in Noto.
3. Else run by run: each run of the label from the first font that has it,
   the role's font first, then the locale's fonts, `UiDesc.fallbackFonts`
   (a Hebrew or Devanagari face, a CJK face cooked with `runtime true`) and
   the style's `fallback`; a letter and its accents stay in one face. A
   CJK-only subset (lantern-vale's `tools/subset-font.py`) gives this: the
   Latin in the style's face, the Chinese in Noto, on one line.

So choose by the subset: keep Latin in the CJK subset for one face per
label, leave it out to keep the game's Latin face beside CJK. Tracking
applies to Latin text only.

## 8. Proving it

`templates/ui-kit` in the engine repository runs every template; its
playtests show how to drive UI from a script: `move_to_text:=shop` then
`touch_start`/`touch_end`, `key_down:Tab`/`key_up:Tab`, a game command that
calls `og.ui.navigate`, `dump_ui` for bounds and texts, and frame comparison
against references. `og polish .` scores the screens' contrast, text size,
transitions and sounds (og-build §5), and runs `og ui check`'s rules on
every frame its playtests capture.

**The loop: preview, check, adjust.** A UI is not done until it has been
seen on every surface and in every language, and the checks are clean:

```sh
# A contact sheet per screen (and style): a row per breakpoint (desktop,
# phone portrait and landscape, tablet), a column per locale (the game's,
# a pseudo-locale 40 % longer with accents and brackets, a CJK one, a
# right-to-left one), with index.json naming what each cell shows.
og ui preview . --screen title --screen hud=show:play
# Every rule, with the element paths, the measured value and the prop to change.
og ui check . --screen title --screen hud=show:play
# A motion as a filmstrip: the screen settles, then the commands, a frame at each time.
og ui preview . --screen hud=show:play --film "damage:30" --at 0,0.05,0.1,0.2,0.4
```

A screen is a name and the commands that reach it (`--screen
hud=show:play`, steps `t:action;...` from the cell's start); a bare
`--screen <name>` runs `show:<name>` (change it with `--show
"open:{screen}"`). `--styles pixel,neon --style-cmd "style:{style}"`
adds a style axis, `--breakpoints` and `--locales` narrow the others.
Without `--screen`, both commands cover every screen the project
declares: project.ogt's `ui { screen "<name>" "<commands>" }` lines (and
its `screens { title ... play ... results ... }` block), else every value
the game's show command lists in its usage (`commandHelp: { show: { usage:
"<title|play|shop>" } }`), else each screen the playtests take a
screenshot of. Declare them, so a bare `og ui check .` sees everything:

```text
ui {
  screen "title" "0:show:title"
  screen "hud" "0:show:play"
  screen "shop" "0:show:play;30:key:KeyS"
}
```

Each line is `screen`, a name and one commands string (join steps with
`;`). The cook refuses a malformed block with its line and fix: a line
that is not `screen`, a screen with no name or no commands, more than one
commands string, a name declared twice. A bare `og ui check .` is the full
review (every screen × style × breakpoint × locale) and `og polish` runs
the same set; a playtest that runs in the gate names its few screens with
`--screen`.
Each cell is a fresh engine laid out through og.ui.previewViewport, so a
game needs no viewport command of its own. Outputs land in
`og-out/ui-check/` (check.json, check.txt; exit 3 on an error) and
`og-out/ui-preview/` (sheet-<screen>.png, index.json, film.png).

What `og ui check` reports (docs/ui.md §16.5), each with the fix:

| Rule | What fails | Change |
|---|---|---|
| `overlap` | two elements over each other that nothing declared (§6.2): placed groups that collide at a breakpoint, siblings in the flow | the placed one's anchor, offset or a breakpoint variant; `overlaps: ["<path>"]` in og.ui.props when it is the design |
| `text-cut`, `offscreen` | text cut by its clip, drawn wider than its box, a control past the screen with no scroll | let the box fit or wrap; `ellipsis` with a tooltip; a scroll container |
| `text-truncated` | an ellipsis without a tooltip (a warning: a tooltip only exists while hovered) | og.ui.tooltip right after it |
| `safe-area`, `covered` | under the notch or home indicator, or under the player shell's Back (`Viewport.covered`) | place with og.ui.anchor, which keeps to both |
| `touch-target` | an interactive element under 44×44 CSS px on a coarse pointer | its size, or hit slop |
| `contrast` | text under 4.5:1 on the pixels (7:1 in high contrast) | the style's palette, never per call |
| `text-size` | text under 12 CSS px on touch, under 18 px at 1080p on desktop | a larger step of the ramp |
| `focus-reach`, `focus-trap` | a control the d-pad cannot reach from the default focus (two in one place, not focusable), or cannot leave | line controls up in rows and columns; make it focusable |
| `a11y-name`, `a11y-locale` | a control with no name; a name or text the same in another locale as in the game's own | its text (or its desc's `label`, where the widget has one); og.i18n.t |
| `primary-actions` | more than one primary button in a layer | all but one `secondary` |
| `feedback` | a styled game without hover and press sounds; a custom interactive container without visual states | the style's sounds; a `press` state group |
| `budget` | over 300 elements in a frame | a virtual list, fewer wrappers |

Not checkable yet, and reported as such: 200 % text scale (og-ui has no
text scale), a layer without Back (the layer stack), motion bounds and
reduced-motion fallbacks (the style's motion tokens on states), art
slots on their placeholder. og.ui.overlaps and og.ui.layoutStats read the
same overlap and counts from game code.

In a playtest, the same checker: take `dump_ui:<name>.uicheck.json` (the
run writes `<name>.uiframe.json` beside it) and read it with `checkUi`:

```ts
import { playtest } from "@origingame/engine/playtest";
import { checkUi } from "@origingame/engine/ui-check";

export default playtest({
  name: "the HUD is clean on a phone",
  commands: "0:show:play;0.8:dump_ui:hud.uicheck.json;0.9:quit",
  check(run, t) {
    const errors = checkUi(run, "hud", { skip: ["contrast"] }).filter((f) => f.severity === "error");
    t.equal(errors.map((f) => `${f.rule} ${f.paths.join(" × ")}`), [], "og ui check finds nothing");
  },
});
```

ui-craft has the design side: hierarchy, each genre's layout grammar and
motion personality.

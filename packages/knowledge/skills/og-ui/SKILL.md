---
name: og-ui
description: Game screens and HUDs on Origin Engine with og.ui — which screen template to pick (title, pause, settings, five HUDs, cutscene, shop, inventory, level select, results, leaderboard, tutorial, loading, dialogs, toasts), how to feed it data and read what it answers, the widget set beneath, focus for keyboard and gamepad, phones and the safe area, and theming from the platform tokens. Use when building or changing menus, HUDs or any UI in a project with a project.ogt.
---

`og.ui` is the engine's immediate-mode UI: a system describes the tree every
tick, the engine lays it out, draws it in the platform's design tokens and
answers the probe (`dump_ui`, `touch_move_to`, `move_to_text`). The API
reference is `reference/ui.md`; this skill is how to use it well.

## The rule of the kit

Every template and widget is **one call per tick, data in, answer out**:

```ts
const r = og.ui.shopScreen("Shop", { balance: run.coins, items: SHOP, categories: ["Gear", "Items"] });
if (r.bought !== null) { /* take the price, grant the item */ }
if (r.closed) state.screen = "map";
```

- Keep game state (open flags, the current line, the settings, slots) in a
  component or resource, pass it in every tick, store what comes back. Never
  keep UI state in module variables.
- What only the UI needs between ticks (hover, focus, scroll, a menu being
  open, the chosen tab of a template, animation progress, toasts) is UI state:
  snapshots and replays carry it. A template that needs a number of its own
  uses `og.ui.memory(id, initial)` / `og.ui.remember(id, value)`.
- Call UI only inside a system. Inside a system the component you write is
  borrowed: read and write it through the archetype's column views, not
  `world.get`/`world.set` (templates/ui-kit/src/main.ts shows the pattern).

## Pick a template

Screenshots of every template at desktop (1280×720) and on a phone in portrait
(390×844 at 3×) are in the engine repository under `templates/ui-kit/screens/`;
the full-size reference frames are `crates/og-ui/references/metal/kit-*.png`.

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
| A shop | `og.ui.shopScreen(id, { balance, currency, items, categories })` | `bought`, `closed` |
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

## Below the templates: the widget set

Layout (`row`, `column`, `panel`, `scroll`, `anchor`), text and controls
(`label`, `button`, `toggle`, `slider`, `textField`, `checkbox`,
`checkboxList`, `radio`, `tabs`, `select`, `stepper`, `chip`, `badge`,
`keyValue`, `divider`, `spacer`, `icon`, `image`, `progress`, `skeleton`,
`spinner`), HUD parts (`meter` for health, mana and xp with a trail that
follows a drop; `cooldown` rings; `minimap`; `dialogue` with typed text),
touch (`joystick`, `dpad`, `actionButton`, each at least 44 CSS px), lists
(`list` and `grid`, virtualised: only rows in view are built; items report
drag and drop), layers (`anchor` for HUD corners, `modal` for dialogs,
`transition` for fade, slide and scale by the tick, `spotlight`) and
`toast`. Ids are stable strings: give every control a meaningful id so a
playtest can `touch_move_to` it (`Shop/Panel/Detail/Buy` resolves as
`Detail/Buy`).

## Focus: keyboard and gamepad

Arrows and the d-pad move focus to the nearest control that way (wrapping at
the edges), Tab and Shift+Tab walk reading order, Enter, Space and A press,
Escape and B go back: the topmost dialog or menu closes, otherwise
`og.ui.back()` answers true once (open pause on it). A gameplay HUD whose keys
move the player calls `og.ui.setNavigation(false)` (the HUD templates do);
dialogs turn navigation back on while open. The UI reads pad 0 from
`og.Gamepad` itself (d-pad, A, B, the left stick); for a custom binding,
queue intents with `og.ui.navigate(world, intent)`. Script a pad in a
playtest with `gamepad:a` then `gamepad:-` (release), or `gamepad:-:0,0.9`
for the stick.

The touch controls (joystick, d-pad, action buttons) answer several fingers
at once: a thumb holding the stick and another on a button both count. Test
it with `touch_start` on the stick and `touch:1:down:<button id>` /
`touch:1:up:<button id>` for the second finger.

The UI is drawn over the finished frame (after bloom, grading and TAA), so
its colours are exactly the tokens and it stays sharp whatever camera the
game uses; a scroll list cuts its rows at the edge per pixel.

## Phones, tablets and the safe area

Anchored groups sit inside the safe area (notch, home indicator) at
`topLeft` … `bottomRight` or `center`, inset by the density's gutter.
`og.ui.frameViewport().breakpoint` is `phonePortrait`, `phoneLandscape`,
`tablet` or `desktop`; templates rearrange themselves by it (tabs become a
select, side panels move under, touch controls appear when `touch` is set).
Preview a phone headless with a game command that calls
`og.ui.previewViewport(world, { width: 390, height: 844, dpr: 3, safeArea: { top: 47, bottom: 34, left: 0, right: 0 }, coarsePointer: true })`
and `og run --size 1170x2532`.

## Theming

The look is the platform's tokens: never pass hex colours for chrome; use
`{ token: "accent" }`, `"fgMuted"`, `"surface"` and the type steps
(`displayLg`, `xl`, `meta`). Configure once in `GameDesc.start`:

```ts
og.ui.configure(world, {
  font,                              // og.load.font
  scheme: "dark",                    // the default: games run on a stage
  accentHue: "blue",                 // or accentColor: { r, g, b }; hover, active, ink and focus are derived
  contrast: "high",                  // stronger inks and fills, outlined controls
  reducedMotion: true,               // transitions only fade
  icons: { texture, columns: 8, rows: 4, names: [...] },  // og.ui.icon(id, "heart")
});
```

Amber (the accent) is for the one primary action per screen, ink and focus;
selection is a neutral fill. Pass a `GameSettings`' `highContrast` and
`reducedMotion` straight into `configure` when they change.

## Proving it

`templates/ui-kit` in the engine repository runs every template; its
playtests show how to drive UI from a script: `move_to_text:=shop` then
`touch_start`/`touch_end`, `key_down:Tab`/`key_up:Tab`, a game command that
calls `og.ui.navigate`, `dump_ui` for bounds and texts, and frame comparison
against references.

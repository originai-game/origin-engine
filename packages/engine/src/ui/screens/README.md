# Built-in screens

TSX screen components and generic data/feedback pieces built from the
public UI primitives. Import a screen by its module path and mount it on
a layer; nothing is installed or loaded as a document:

```tsx
import { mount } from "@origingame/engine/ui";
import { PauseScreen } from "@origingame/engine/ui/screens/pause.tsx";

// In the game's UI registration, with world supplied by the engine:
const close = mount(world, PauseScreen, {
  objective: "",
  played: 0,
  onResume: () => { /* Request resume through game state. */ },
}, { layer: "menu" });
```

Call the returned disposer to close the screen. The layer sets draw/input
ordering; the game still owns pause and screen state. `mount` does not
read the screen's exported `screen` metadata as a gameplay instruction.
The contract and acceptance rules are in `docs/ui.md`.

## A screen module

- An exported `<Name>Props` interface documents inputs and callbacks.
  The component body runs once; reactive values are read in JSX/accessors.
- Props and function-valued content (`side`, `hint`, `children`, as the
  component declares) replace named stores/snippets in document markup.
  Use callback props (`onResume`, `onChoose`) rather than named actions in
  string attributes. Controlled values use value/change callbacks.
- Visible words use `t(key, params)` and companion string tables imported
  beside TSX (`import "./pause.en.strings.ts"`), not `<meta>` headers. Locale
  formatters handle numbers and durations.
- `fixtures` exports named prop sets, merged over `default`, for the
  gallery. Screens can also export `screen` metadata describing their
  preview identity; it is not an implicit document registry.
- Look is class strings and `packClass` slots. There are no HTML hosts,
  scoped CSS rules, `::part` selectors or `.svelte` loading paths.

`og ui eject screens/pause` copies TSX to `src/ui/screens/` and locale tables to
`content/ui/screens/`, prefixes copied message keys with `game.`, and
rewrites matching imports in the game's code. New imports must name the
copy. It does not globally override engine-internal imports. Existing
files are refused without `--force`.

## Rows

Built-in action rows/buttons generally appear only when the corresponding
callback is passed: Pause without `onLoad` has no Load row, MainMenu
without `onModes` has no Modes row, Title without `onStart` no prompt.
A temporarily unavailable action stays drawn but grey through its state
prop: `canSave` for Save, `pending` for Apply/Revert. Data lists remain
visible without handlers. Check the specific component for its contract.

Pause accepts game rows (`rows`, each `{ key, label, after?, minor?,
danger?, disabled? }`) and reports `onRow(event, key)`. Supply that
handler when passing custom actions. Stable keys identify rows across
reorders; `For` item/index arguments are accessors, not plain objects.

## Placement and tutorial targets

Full-screen screens use `absolute inset-0` to fill their layer or the
parent a game composes them inside. A supported `class` prop follows the
screen's own classes for placement/insets. Content taller than the layer
must scroll without losing its title or actions. Built-in screens apply
their own `env(hud-inset-*)` safe-area padding; do not wrap them in the
same inset again. Backgrounds may extend to the surface edges.

Tutorial's `target` is a UI node's `name`, not an HTML/document id. The
spotlight dims and intercepts presses outside that node, leaving it lit
and interactive. The card stands below it; while no target shows it
falls back to the supplied `x`, `y`. The game owns any pause/input rule
while waiting for the requested action.

## Catalogue

Every path below is a `.tsx` module under this directory; its exported
props and fixtures are authoritative. Use `og ui eject --list` for the
current copyable paths and the gallery for rendered fixtures:
`bun packages/engine/src/node/gallery.ts --only screens/`.

| Purpose | Modules and exports |
| --- | --- |
| Boot, splash, title, main menu, mode select | `splash` → `SplashScreen`, `title` → `TitleScreen`, `main-menu` → `MainMenuScreen`, `mode-select` → `ModeSelectScreen` |
| Settings, controls/remapping, HUD layout | `settings` → `SettingsScreen`, `controls` → `ControlsScreen`, `hud-editor` → `HudEditorScreen` |
| Pause/system menu | `pause` → `PauseScreen` |
| Save/load with thumbnails | `save-load` → `SaveLoadScreen` |
| Loading tips and progress | `loading` → `LoadingScreen` |
| Results/post-match | `results` → `ResultsScreen` |
| Leaderboards | `leaderboard` → `LeaderboardScreen` |
| Calibration | `calibration` → `CalibrationScreen` |
| Tutorial/onboarding | `tutorial` → `TutorialScreen` |
| Death/retry/revive/game over | `game-over` → `GameOverScreen` |
| Creator tools | `creator-tools` → `CreatorToolsScreen` |
| Credits | `credits` → `CreditsScreen` |
| Interactive film picture, captions and choices | `flow-screen` → `FlowPicture`, `FlowScreen`; `flow-choices` → `FlowChoices` |
| Film chapters, flowchart and ending gallery | `flow-chapters` → `FlowChaptersScreen`, `flow-chart` → `FlowChartScreen`, `flow-gallery` → `FlowGalleryScreen` |
| Card/confirmation | `card` → `Card`, `confirm` → `Confirm` |
| Lists, key-values, tables and trees | `data-list` → `DataList`, `key-values` → `KeyValues`, `table` → `Table`, `tree` → `Tree` |
| Calendar/pagination | `calendar` → `Calendar`, `pagination` → `Pagination` |
| Objectives/rating/banner | `objective-tracker` → `ObjectiveTracker`, `star-rating` → `StarRating`, `banner` → `Banner` |
| Subtitles/notifications | `subtitles` → `Subtitles`, `notifications` → `NotificationsScreen` |

## Styling surfaces

Customize the shared `Screen.frame`, `Panel.frame`, `Plate.frame`,
`Row.frame`, `Head.frame`, `Button.plate`, `Bar.*` and other slots actually
named by the source. States are props and boolean `data-*` variants;
focus/hover/active/disabled styles come from the same controls used by
components elsewhere. A source's `name` prop is a probe/anchor identity,
not a CSS selector or a declaration of an externally addressable part.
Change private layout by ejecting the component, not by targeting its
descendants with selectors.

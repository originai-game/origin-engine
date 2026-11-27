<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PauseScreen"></a>
### `PauseScreen`

The pause menu: Resume, Settings, Controls, Save (grey when not allowed), Load, Quit to menu and Quit game, each drawn only when the game passes its handler, and the game's own rows among them, with the current objective and the time played beside it. Each reports its event.

`import { PauseScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/pause.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `objective` | `string` | The current objective (empty: none). |
| `played` | `number` | Seconds played this run. |
| `canSave?` | `boolean` | Saving is allowed here. |
| `rows?` | `ReadonlyArray<PauseRow>` | The game's own rows (Restart stage), each drawn as a built-in row and reported by onRow with its key: in the main group after the built-in ones, or after the row `after` names (a built-in row's key: resume, settings, controls, save, load, quitToMenu, quit; or an earlier row of the game's); a `minor` row goes in the group set apart below them, with Quit to menu and Quit game. |
| `onRow?` | `((e: UiEvent, key: string) => void) \| undefined` | A row of the game's (props.rows) pressed, with its key. |
| `side?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the side snippet). |
| `onControls?` | `((e: UiEvent, key?: string) => void) \| undefined` | The controls event, with the row's key where it comes from a row. |
| `onLoad?` | `((e: UiEvent, key?: string) => void) \| undefined` | The load event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onQuitToMenu?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quitToMenu event, with the row's key where it comes from a row. |
| `onResume?` | `((e: UiEvent, key?: string) => void) \| undefined` | The resume event, with the row's key where it comes from a row. |
| `onSave?` | `((e: UiEvent, key?: string) => void) \| undefined` | The save event, with the row's key where it comes from a row. |
| `onSettings?` | `((e: UiEvent, key?: string) => void) \| undefined` | The settings event, with the row's key where it comes from a row. |

Each `PauseRow`:

| Field | Type | What |
|---|---|---|
| `key` | `string` | Its stable key, reported by onRow. |
| `label` | `string` | What it says. |
| `after?` | `string` | The key of the row it follows; empty or unknown: the end of its group. |
| `minor?` | `boolean` | In the group set apart below, with Quit to menu and Quit game, without a chevron. |
| `danger?` | `boolean` | Tinted as a danger when hovered, as Quit game is. |
| `disabled?` | `boolean` | Drawn grey and takes no press. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`, `ownRows`.

Actions: `confirm` (Select), `back` (Resume).

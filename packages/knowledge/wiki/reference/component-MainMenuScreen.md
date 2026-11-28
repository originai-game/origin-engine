<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-MainMenuScreen"></a>
### `MainMenuScreen`

The main menu over the key art: the logo or the name as a wordmark, Continue with the last save's line when a save exists, New game, Modes, Settings, Credits and Quit, each drawn only when the game passes its handler, a news card (pressable with onNews) and the version. It resolves in the dark scheme in both schemes.

`import { MainMenuScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/main-menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `canContinue?` | `boolean` | A save exists: Continue shows and takes the focus. |
| `saveLine` | `string` | Under Continue: where the last save stands (chapter, time played). |
| `version` | `string` | The build's version, shown small in the corner. |
| `news` | `string` | A line for the news card (empty: no card). |
| `keyArt` | `Texture \| 0` | The key art's texture behind the menu (og.load.texture; 0: the pack's designed backdrop, Screen.backdrop). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. |
| `logo` | `Texture \| 0` | The game's logo texture (og.load.texture; 0: the name set as a wordmark). |
| `name` | `string` | The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). |
| `background?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the background snippet). |
| `extra?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the extra snippet). |
| `onContinue?` | `((e: UiEvent, key?: string) => void) \| undefined` | The continue event, with the row's key where it comes from a row. |
| `onCredits?` | `((e: UiEvent, key?: string) => void) \| undefined` | The credits event, with the row's key where it comes from a row. |
| `onModes?` | `((e: UiEvent, key?: string) => void) \| undefined` | The modes event, with the row's key where it comes from a row. |
| `onNewGame?` | `((e: UiEvent, key?: string) => void) \| undefined` | The newGame event, with the row's key where it comes from a row. |
| `onNews?` | `((e: UiEvent, key?: string) => void) \| undefined` | The news event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onSettings?` | `((e: UiEvent, key?: string) => void) \| undefined` | The settings event, with the row's key where it comes from a row. |

Pack slots: `MenuList.frame`, `Plate.frame`, `Screen.frame`, `Screen.note`, `Title.wordmark`.

Fixtures: `default`.

Actions: `confirm` (Select).

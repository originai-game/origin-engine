<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SaveLoadScreen"></a>
### `SaveLoadScreen`

Saving or loading: the slots with their title, place, time played, date and thumbnail (autosaves marked, empty slots offered for a new save), the selected one's preview, and Save or Load and Delete. The game writes the saves.

`import { SaveLoadScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/save-load.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `mode` | `string` | save or load: the title and the main action. |
| `slots` | `ReadonlyArray<{ auto: boolean; date: string; empty: boolean; id: string; place: string; played: number; thumb: Texture \| 0; title: string }>` | The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }. |
| `selected` | `string` | The stable collection key selected now. |
| `preview?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the preview snippet). |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onDelete?` | `((e: UiEvent, key?: string) => void) \| undefined` | The delete event, with the row's key where it comes from a row. |
| `onLoad?` | `((e: UiEvent, key?: string) => void) \| undefined` | The load event, with the row's key where it comes from a row. |
| `onNewSlot?` | `((e: UiEvent, key?: string) => void) \| undefined` | The newSlot event, with the row's key where it comes from a row. |
| `onSave?` | `((e: UiEvent, key?: string) => void) \| undefined` | The save event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Select), `back` (Back), `details` (Delete).

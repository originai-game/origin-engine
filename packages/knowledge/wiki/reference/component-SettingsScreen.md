<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SettingsScreen"></a>
### `SettingsScreen`

Settings: a tab per page and its entries (toggles, ranges, choices and key bindings), Apply and Revert while changes wait, and Defaults. The game owns every value and writes back what the events report.

`import { SettingsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/settings.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `note` | `string` | A line under the title (Some changes apply after a restart); empty: none. |
| `pages` | `ReadonlyArray<{ id: string; name: string; icon?: string }>` | The settings' pages as a keyed collection of { id, name, icon? }: a tab each, with the icon (an icon set's name) before its name. |
| `entries` | `ReadonlyArray<{ id: string; page: string; name: string; description: string; kind: string; value: boolean \| number \| string; min?: number; max?: number; step?: number; options?: ReadonlyArray<string>; action?: string }>` | The settings as a keyed collection of { id, page, name, description, kind: toggle\|range\|select\|binding, value, min, max, step, options, action }, shown on their page's tab; each value is written back as the player changes it (two-way via onChange). |
| `page` | `string` | The id of the page shown (two-way via onPage: the game writes it back as the player picks a tab); empty: no tree of pages, the `tree` content alone. |
| `pending?` | `boolean` | Changes wait to be applied: Apply and Revert take presses. |
| `tree?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the tree snippet). |
| `onPage?` | `(e: UiEvent, id: string) => void` | The page tab picked, with the tab's id; the game writes props.page back. |
| `onChange?` | `(e: UiEvent, id: string, value: boolean \| number \| string) => void` | An entry's value changed, with the entry's id and its new value; the game writes entries' value back. |
| `onRebind?` | `(e: UiEvent, id: string, key: string) => void` | The key captured while a binding row listens, with the entry's id and the key; the game writes entries' value back. Escape cancels without calling this. |
| `onApply?` | `((e: UiEvent, key?: string) => void) \| undefined` | The apply event, with the row's key where it comes from a row. |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onDefaults?` | `((e: UiEvent, key?: string) => void) \| undefined` | The defaults event, with the row's key where it comes from a row. |
| `onRevert?` | `((e: UiEvent, key?: string) => void) \| undefined` | The revert event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Screen.frame`, `Screen.heading`, `Slot.frame`.

Fixtures: `default`.

Actions: `confirm` (Select), `back` (Back), `tabNext` (Next page), `tabPrev` (Previous page).

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Menu"></a>
### `Menu`

A menu: a dense smoked plate with a lit top edge, its rows dim type that wash in the accent from a lit leading edge under the pointer, the keys or the focus, a shortcut on a dark key chip, a check drawn in the accent, the chosen radio a lit diamond, a destructive command in the danger ink and a disabled row faint. A submenu's rows show in place of the menu's under a back row, and a header or a separator parts the entries. Reports choose (an entry's key) and back.

Dropped against the .svelte: the CJK heading size (a lang has no class variant here; the engine's text scale carries it).

`import { Menu } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `entries` | `ReadonlyArray<{ id: string; label: string; kind: string; icon: string; shortcut: string; checked: boolean; danger: boolean; disabled: boolean; parent: string; more: boolean }>` | Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end). |
| `submenu` | `string` | The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level. |
| `label` | `string` | What a screen reader calls it; empty is Menu. |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`.

Fixtures: `default`.

Actions: `choose` (Choose), `back` (Back).

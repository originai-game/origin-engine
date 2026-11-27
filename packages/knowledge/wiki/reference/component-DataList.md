<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DataList"></a>
### `DataList`

A HUD plate of rows: a name, its detail and its figure.

`import { DataList } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/data-list.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<{ id: string; title: string; detail: string; value: string }>` | The rows, a keyed collection of { id, title, detail, value }. |
| `loading?` | `boolean` | Rows are on their way: a busy line instead of the empty one. |
| `empty` | `string` | What an empty list says. |
| `busy` | `string` | What a loading list says. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `choose` (Open).

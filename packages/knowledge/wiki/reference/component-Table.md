<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Table"></a>
### `Table`

A HUD plate of plain rows: the first column a name, the others figures.

`import { Table } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/table.tsx`).

| Prop | Type | What |
|---|---|---|
| `columns` | `ReadonlyArray<{ id: string; label: string }>` | The headings, a keyed collection of { id, label }. |
| `cells` | `ReadonlyArray<{ id: string; row: string; text: string }>` | The body's cells row by row, a keyed collection of { id, row, text }: three to a row (eject it to change the columns). A press sends choose with the cell's id; its row field names the row. |
| `label` | `string` | The table's accessible name. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `choose` (Open row).

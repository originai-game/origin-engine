<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DataGrid"></a>
### `DataGrid`

The HUD's smoked-glass plate: headings in small tracked capitals over a faint rule, rows dense and plain so the data reads first, figures condensed and tabular. The chosen row is washed in the accent from its leading edge. In a narrow shell each row becomes a two-line card: the first column and the last on one line, the rest as a caption under. Reports choose (a row's key) and retry.

Each row is its own plate; the chosen row's wash is one flat colour.

`import { DataGrid } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/data-grid.tsx`).

| Prop | Type | What |
|---|---|---|
| `columns` | `ReadonlyArray<{ id: string; label: string; numeric: boolean }>` | The headings, a keyed collection of { id, label, numeric }: numeric columns align to the end in tabular figures. |
| `tracks` | `string` | The columns' widths as CSS grid tracks, one per column; empty shares the width equally between four. A numeric column's track holds its widest figure in text-scaled rem units (e.g. 6rem), so it grows with the player's text size; a figure is never cut, and the text columns (fr) give way. |
| `rows` | `ReadonlyArray<{ id: string; cells: ReadonlyArray<{ id: string; text: string; numeric: boolean }> }>` | The rows, a keyed collection of { id, cells }: cells a keyed list of { id, text, numeric }, in the columns' order. Only the rows in view are built. |
| `selected` | `string` | The id of the selected row; empty is none. |
| `label` | `string` | What a screen reader calls the grid. |
| `loading?` | `boolean` | Rows are on their way: busy placeholder rows under the header. |
| `error` | `string` | An error to show with Retry instead of the rows; empty is none. |
| `empty` | `string` | What an empty grid says; empty is Nothing here yet. |
| `hint` | `string` | A line under the empty message. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `loading`, `error`, `empty`, `selected`.

Actions: `choose` (Select), `retry` (Retry).

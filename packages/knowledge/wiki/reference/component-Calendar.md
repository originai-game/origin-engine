<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Calendar"></a>
### `Calendar`

A HUD plate: the month between cut arrow keys, the days dark recesses, today lit in the accent and a marked day carrying a gold diamond.

`import { Calendar } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/calendar.tsx`).

| Prop | Type | What |
|---|---|---|
| `month` | `string` | The month shown (March 2027). |
| `weekdays` | `ReadonlyArray<{ id: string; label: string }>` | The seven weekday headings, a keyed collection of { id, label }. |
| `days` | `ReadonlyArray<{ id: string; label: string; today: boolean; outside: boolean; marked: boolean }>` | The cells, a keyed collection of { id, label, outside, today, marked } from the week's first day (outside: another month's). |
| `previousLabel?` | `string \| undefined` | Previous's accessible name (Previous month); absent: the month. |
| `nextLabel?` | `string \| undefined` | Next's accessible name (Next month); absent: the month. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPick?` | `(e: UiEvent, key?: string) => void` | The pick event, with the row's key where it comes from a row. |
| `onPrevious?` | `(e: UiEvent, key?: string) => void` | The previous event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `previous` (Previous month), `next` (Next month), `pick` (Choose day).

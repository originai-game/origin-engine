<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Chart"></a>
### `Chart`

A HUD plate: the title under an accent kicker, bars standing on a hairline grid with the axis in faint tabular figures, a line series in gold over them through each bar's middle, and each bar's name under it (every other name when there are more than eight). Reports nothing.

`import { Chart } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/chart.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | A title over it; empty is none. |
| `subtitle?` | `string` | A line under the title. |
| `max?` | `number` | The value axis' top (its bottom is 0). Default 1. |
| `bars` | `ReadonlyArray<{ id: string; label: string; value: number; tone: string }>` | Bars, one per category, left to right: each { id, label, value, tone }, tone a role (danger, success, warn, info, gold) to set one apart, empty the accent. |
| `line?` | `readonly number[]` | A line series over the bars, one value per bar (on the same axis); empty is none. |
| `ticks?` | `ReadonlyArray<{ id: string; at: number; label: string }>` | Value axis gridlines: each { id, at, label }, at a fraction of max (0 to 1). |

Pack slots: `Panel.frame`.

Fixtures: `default`, `dense`, `empty`.

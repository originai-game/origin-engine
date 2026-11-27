<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ProgressRing"></a>
### `ProgressRing`

A ring that fills with its value, its percent (or the label's words) in the middle; with no known value the ring turns instead. Its side, track, fill, spin and label are the Ring component's pack slots. Read only.

`import { ProgressRing } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/progress-ring.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | Done, 0 to 1. |
| `spinning?` | `boolean` | Working with no known value: the arc turns (indeterminate). |
| `label?` | `string` | Text in the middle; empty is the percent (none while spinning). |
| `name?` | `string` | What a screen reader calls it; empty is Progress, or Loading while spinning. |
| `tone?` | `string` | The fill's tone: primary (empty), success, warn, danger or info. |

Pack slots: `Ring.box`, `Ring.fill`, `Ring.fill.…`, `Ring.label`, `Ring.rest`, `Ring.spin`, `Ring.track`.

Fixtures: `default`, `spinning`, `labelled`, `danger`.

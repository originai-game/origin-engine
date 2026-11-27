<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ProgressBar"></a>
### `ProgressBar`

A bar fills the width its container gives it: a caption over it at the start and its value in words at the end, a paler buffer ahead of the fill in a dark well with one lit rim, thinner than a vital's; with no known value a band sweeps the track instead. It sits on a HUD plate, so its words read over any scene. The style's lean (skewX) and the fill's sheen have no TSX surface: the lean is dropped and the sweeping band carries the shimmer.

`import { ProgressBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/progress-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes merged after its own: a bar inside another surface takes the next layer (`bg-layer-3`). |
| `value` | `number` | Done, 0 to 1. |
| `buffer?` | `number` | A paler fill ahead of value (downloaded ahead of installed), 0 to 1. |
| `label?` | `string` | A caption over it at the start; empty is none. |
| `valueText?` | `string` | The value in words at the end; empty is the percent. |
| `showValue?` | `boolean` | Show the value at the end. |
| `indeterminate?` | `boolean` | Working with no known value: a band sweeps along the track. |
| `tone?` | `string` | The fill's tone: primary (empty), success, warn, danger or info. |

Pack slots: `Bar.fill`, `Plate.frame`.

Fixtures: `default`, `indeterminate`, `success`, `danger`.

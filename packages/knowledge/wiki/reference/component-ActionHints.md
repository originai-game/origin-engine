<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ActionHints"></a>
### `ActionHints`

A bar of input hints: each a key cap showing its action's binding glyph and what it does, one that fires on a hold keeping the hold and running its fill as it holds.

`import { ActionHints } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/action-hints.tsx`).

| Prop | Type | What |
|---|---|---|
| `hints?` | `ReadonlyArray<{ action: string; hint: string; hold?: boolean }>` | The hints, a keyed collection of { action, hint, hold }: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it). |
| `label?` | `string` | What a screen reader calls the bar; empty is Actions. |
| `onPress?` | `(e: UiEvent, action: string) => void` | Do. |
| `onHold?` | `(e: UiEvent, action: string) => void` | Hold, completing after 0.8 s; the hold hint's fill runs while it holds. |

Fixtures: `default`.

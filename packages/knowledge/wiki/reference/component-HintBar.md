<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HintBar"></a>
### `HintBar`

The footer of input hints a full-screen built-in shows (A Select · B Back): each action's glyph on the device used last, then its words, in the pack's hint type (Hint.bar, Hint.item). Hidden on touch, where no button is pressed. Purely presentational.

`import { HintBar } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/hint-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `hints` | `ReadonlyArray<{ action: string; label: string }>` | The hints, in order: each an input action (confirm, back, tabNext) and the words for what it does here. |
| `class?` | `string \| undefined` | Classes after its own (its placement). |

Pack slots: `Hint.bar`, `Hint.item`.

Fixtures: `default`.

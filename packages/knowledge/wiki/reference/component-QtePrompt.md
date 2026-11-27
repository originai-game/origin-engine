<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-QtePrompt"></a>
### `QtePrompt`

A keyed prompt over the scene: what to attempt, a big trigger (press, hold or mash — the hold runs holdMs with its fill, and confirms when it completes), its state in words, and the game-supplied time left (and, for a mash, the response progress) as thin bars. Only the active state accepts input; the game owns resolution and expiry. Read only beyond the respond.

`import { QtePrompt } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/qte-prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The action to attempt; empty uses Respond. |
| `keyLabel?` | `string` | The displayed input hint, not an input binding; empty hides it. |
| `action?` | `string` | An og-input action bound by the game. Pointer, focused keyboard/pad and this action activate the same control. |
| `mode?` | `string` | press, hold or mash. Hold emits once after holdMs; mash emits one respond per activation. The game counts responses and judges success. |
| `holdMs?` | `number` | The minimum hold duration in milliseconds, at least 1. Releasing early cancels without an intent. |
| `remaining?` | `number` | Time left as a fraction, 0 to 1. The game advances it; no clock runs here and zero does not choose a result. |
| `progress?` | `number` | Game-supplied completion fraction, 0 to 1; displayed for mash prompts. |
| `state?` | `string` | active, waiting, success, failure or expired. Only active accepts input; the game owns resolution and expiry. |
| `disabled?` | `boolean` | Temporarily unavailable even while active. |
| `onRespond?` | `(e: UiEvent) => void` | Respond; the game owns resolution. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `hold`, `mash`, `success`, `failure`, `expired`, `waiting`, `disabled`, `overflow`.

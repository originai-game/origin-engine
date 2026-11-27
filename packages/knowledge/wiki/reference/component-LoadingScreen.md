<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-LoadingScreen"></a>
### `LoadingScreen`

The key art fills the screen. At its foot, over a shade that grows with the words: where the player is going (an eyebrow and the place), the tip on a cut smoked-glass plate, and the progress as the HUD's leaning bar with a spinner and the figure; once ready, Continue (or nothing, when the game does not wait). A press anywhere asks for the next tip.

`import { LoadingScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/loading.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `keyArt` | `Texture \| 0` | The key art's texture (og.load.texture; 0: the style's placeholder). |
| `place` | `string` | Where the player is going. |
| `tip` | `string` | A tip or a line of lore. |
| `progress` | `number` | 0 to 1. |
| `ready?` | `boolean` | Loading is done. |
| `waits?` | `boolean` | The game waits for a press once ready. |
| `status?` | `string` | What the wait is for, beside the figure while it is not ready (Waiting for the server); empty: none. |
| `lore?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the lore snippet). |
| `onContinue?` | `(e: UiEvent, key?: string) => void` | The continue event, with the row's key where it comes from a row. |
| `onNextTip?` | `(e: UiEvent, key?: string) => void` | The nextTip event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Continue).

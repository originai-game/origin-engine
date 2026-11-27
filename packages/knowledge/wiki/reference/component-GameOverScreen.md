<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-GameOverScreen"></a>
### `GameOverScreen`

Game over: the defeat's line, the run's score and the best, an optional revive offer with its price and the seconds it stays, and Retry, Checkpoint and Quit. The game decides the offer and what each choice costs.

`import { GameOverScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/game-over.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `title` | `string` | The defeat's line. |
| `score` | `number` | The run's score. |
| `best` | `number` | The best score. |
| `offer?` | `boolean` | A revive is offered. |
| `price` | `string` | The revive's price, as text (50 gems, Watch an ad). |
| `left` | `number` | Seconds the offer stays. |
| `checkpoint?` | `boolean` | A checkpoint exists. |
| `offerArt?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the offerArt snippet). |
| `stats?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the stats snippet). |
| `onCheckpoint?` | `((e: UiEvent, key?: string) => void) \| undefined` | The checkpoint event, with the row's key where it comes from a row. |
| `onDecline?` | `((e: UiEvent, key?: string) => void) \| undefined` | The decline event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onRetry?` | `((e: UiEvent, key?: string) => void) \| undefined` | The retry event, with the row's key where it comes from a row. |
| `onRevive?` | `((e: UiEvent, key?: string) => void) \| undefined` | The revive event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Retry).

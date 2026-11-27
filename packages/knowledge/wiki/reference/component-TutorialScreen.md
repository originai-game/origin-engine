<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-TutorialScreen"></a>
### `TutorialScreen`

A tutorial step: a card with the step's title, what to do, its art and the step count, and Next (hidden while the step waits for the player) and Skip. Given a target that shows, the screen dims around that element, which keeps its own presses, and the card stands under it; else (no target, or one not on screen) nothing dims and the card stands where the game puts it.

`import { TutorialScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/tutorial.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The step's title. |
| `text` | `string` | What to do. |
| `step` | `number` | The step, from 1. |
| `steps` | `number` | Steps in the lesson. |
| `waits?` | `boolean` | The step waits for the player to act (Next hidden). |
| `x` | `number` | The card's start edge, CSS px from the screen's start (the right, right to left). |
| `y` | `number` | The card's top, CSS px. |
| `texture` | `Texture \| 0` | The step's art (0: none). |
| `target?` | `string` | The `name` of the element the step points at, anywhere in the UI (name a `View` around an `Icon`): the rest of the screen dims and takes every press, the element stays lit and takes its own, and the card stands under it in place of x and y. Empty: nothing is dimmed and the card stands at x and y. While no element of that name shows, nothing is dimmed and the card stands at x and y. |
| `highlight?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the highlight snippet). |
| `hint?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the hint snippet). |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`, `spotlight`, `targetMissing`.

Actions: `confirm` (Next), `back` (Skip).

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Confirm"></a>
### `Confirm`

A confirmation dialog on its own panel over the pack's scrim, which dims the screen while the panel pops in: the question as its heading, what follows from it, and Cancel and Confirm (the danger variant when confirming destroys something). The game closes it.

`import { Confirm } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/confirm.tsx`).

| Prop | Type | What |
|---|---|---|
| `open?` | `boolean` | Shown; the game writes it false to close (onCancel does on a dismissal). |
| `title` | `string` | Its question, the dialog's heading and accessible name. |
| `body` | `string` | What follows from it; empty: none. |
| `confirm` | `string` | The confirming button's words (Quit, Delete). |
| `cancel` | `string` | The cancelling button's words. |
| `danger?` | `boolean` | Confirming destroys something: the confirm button is the danger variant. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |

Pack slots: `Modal.frame`, `Modal.scrim`, `Panel.frame`.

Fixtures: `default`, `overflow`.

Actions: `confirm` (Confirm), `cancel` (Cancel).

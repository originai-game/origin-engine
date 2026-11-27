<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ToastStack"></a>
### `ToastStack`

Short messages that come and go at the top of the screen (NSH base's `ToastStack` and `Toast`): each is announced to screen readers as it arrives (a `status`, a danger one an `alert`), enters and leaves by the pack's `Toast.frame`, and a swipe sideways/up or the dismiss button closes it. The stack (`Toast.stack`) clears the top safe area and passes presses through between toasts. A game puts it on the `toast` layer.

`import { ToastStack } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/toast.tsx`).

| Prop | Type | What |
|---|---|---|
| `toasts` | `readonly ToastItem[]` | The toasts shown, newest last. The game owns the list and its timing: it drops a toast after its time, or when `onDismiss` reports a swipe or dismiss click. |
| `class?` | `string \| undefined` | Classes of the document that uses it, on the stack. |
| `onDismiss?` | `((e: UiEvent, id: string) => void) \| undefined` |  |

Each `ToastItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | Its key: a toast keeps its node while it stays in the list. |
| `title` | `string` | What it says. |
| `description?` | `string \| undefined` | A second, quieter line. |
| `tone?` | `"success" \| "danger" \| undefined` | Its tone: plain, `success` or `danger` (the pack's `Toast.frame.<tone>`). |

Pack slots: `Toast.frame.…`, `Toast.stack`.

Fixtures: `default`.

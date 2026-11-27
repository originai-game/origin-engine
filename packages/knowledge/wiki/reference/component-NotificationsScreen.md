<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-NotificationsScreen"></a>
### `NotificationsScreen`

The notification stack in the HUD's top-right corner. A notification is a glass plate with two corners cut, a rail down its leading edge and a wash from it in its tone, the tone's mark in an octagon socket, its title in small capitals of the tone and its body in white, and a quiet close. A toast is the compact form: its mark and its words on a plate that slides back out when it leaves. Each tone has its own hue (warn an amber well apart from gold). Under reduced motion they appear in place.

`import { NotificationsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/notifications.tsx`).

| Prop | Type | What |
|---|---|---|
| `items` | `ReadonlyArray<{ id: string; title: string; body: string; tone: string }>` | The game's own notifications, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out. The toasts follow them. |
| `toasts?` | `ReadonlyArray<{ id: string; tone: string; text: string; leaving?: boolean }>` | The game's toasts, as its own keyed rows of { id, tone (info, success, warn, danger), text, leaving (its leave is playing): the game removes one when it leaves and a press dismisses it. |
| `onDismiss?` | `(e: UiEvent, key?: string) => void` | The dismiss event, with the row's key where it comes from a row. |
| `onDismissToast?` | `(e: UiEvent, key?: string) => void` | The dismissToast event, with the row's key where it comes from a row. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

Actions: `dismiss` (Dismiss).

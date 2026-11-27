<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Dialog"></a>
### `Dialog`

A dialog over a scrim (NSH base's `Dialog`; an alert dialog is one with buttons and no other content): modal for screen readers and focus, it pops in by `Modal.frame` over `Modal.scrim`. The pack draws its panel (`Dialog.frame`), heading (`Dialog.title`), text (`Dialog.body`) and button row (`Dialog.actions`). It fills its parent, so a game puts it on the `modal` layer or over the screen it covers.

`import { Dialog } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/dialog.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Shown or not. The game owns it: Escape, `back` or a press on the scrim reports `onClose`, and the game hides it. |
| `title?` | `string \| undefined` | Its heading, which screen readers name it by. |
| `description?` | `string \| undefined` | What it says under the heading. |
| `dismissible?` | `boolean \| undefined` | A press on the scrim closes it (default true); a dialog that must be answered sets false. |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its panel. |
| `onClose?` | `((e: UiEvent) => void) \| undefined` |  |
| `actions?` | `any` | The buttons, in a row at its foot. |
| `children?` | `any` | What it holds between its text and its buttons. |

Pack slots: `Dialog.actions`, `Dialog.body`, `Dialog.frame`, `Dialog.title`, `Modal.frame`, `Modal.scrim`.

Fixtures: `default`.

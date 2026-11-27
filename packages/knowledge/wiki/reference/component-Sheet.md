<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Sheet"></a>
### `Sheet`

A panel that rises from the bottom of the screen over a scrim (NSH base's `Sheet`, the phone's form of a menu or a picker): modal for screen readers and focus. The pack draws its panel (`Sheet.frame`, which slides in and out) and its handle (`Sheet.grabber`). It fills its parent, so a game puts it on the `modal` layer.

`import { Sheet } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/sheet.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Shown or not. The game owns it: Escape, `back`, a press on the scrim or a swipe down reports `onClose`. |
| `label?` | `string \| undefined` | What screen readers call it. |
| `grabber?` | `boolean \| undefined` | The handle at its top (default true). |
| `dismissible?` | `boolean \| undefined` | A press on the scrim, `back`, or a swipe down closes it (default true). |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its panel. |
| `onClose?` | `((e: UiEvent) => void) \| undefined` |  |
| `footer?` | `any` | What sits at its foot, under its content (the main action). |
| `children?` | `any` |  |

Pack slots: `Modal.scrim`, `Sheet.frame`, `Sheet.grabber`.

Fixtures: `default`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ContextMenu"></a>
### `ContextMenu`

The same menu opened by the secondary button (a right click), a finger held on its trigger, or Shift+F10 and the Menu key while the focus is in it (NSH base's `ContextMenu`), at the pointer for mouse/touch or under the trigger for keyboard, with focus on its first item; the trigger's own presses keep working. A game that opens it otherwise sets `open` itself.

`import { ContextMenu } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Shown or not. The game owns it: a right click, a finger held, Shift+F10 or the Menu key on the trigger reports `onOpenChange` with true; choosing an item, a press outside and `back` report false. |
| `label?` | `string \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its popup. |
| `onOpenChange?` | `((e: UiEvent, open: boolean) => void) \| undefined` |  |
| `trigger?` | `any` | What it is the menu of (an item's slot, a unit's card); its own presses keep working. |
| `children?` | `any` |  |

Pack slots: `Menu.frame`, `Menu.indicator`, `Menu.item`, `Menu.item.danger`, `Menu.label`, `Menu.separator`, `Menu.shortcut`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `closed`.

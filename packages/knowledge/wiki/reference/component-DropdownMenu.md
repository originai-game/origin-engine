<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DropdownMenu"></a>
### `DropdownMenu`

A list of actions its trigger opens (NSH base's `Menu`; a dropdown menu on a button): the trigger says it is expanded, and the popup follows the trigger's box, shifted inside the screen. It takes the focus while open, so the arrows move among its items (one focus group that wraps) and closing gives the focus back to the trigger; choosing an item, a press outside it and `back` close it. The pack draws the popup (`Menu.frame`), which pops in by its own `starting:`.

`import { DropdownMenu } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Shown or not. The game owns it: a press on the trigger reports `onOpenChange` with the next value; choosing an item, a press outside and `back` report false. |
| `placement?` | `"bottom" \| "top" \| undefined` | Where it opens by its trigger: `bottom` (the default) or `top`. |
| `label?` | `string \| undefined` | What screen readers call it ("Account"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its popup. |
| `onOpenChange?` | `((e: UiEvent, open: boolean) => void) \| undefined` |  |
| `trigger?` | `any` | What opens it: a control its presses go to (a `Button`, a hero's portrait). |
| `children?` | `any` | Its `MenuItem`s, `MenuSeparator`s and `MenuLabel`s, arranged freely. |

Pack slots: `Menu.frame`, `Menu.indicator`, `Menu.item`, `Menu.item.danger`, `Menu.label`, `Menu.separator`, `Menu.shortcut`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `closed`.

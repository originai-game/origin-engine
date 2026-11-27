<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Button"></a>
### `Button`

A press target (NSH base's `Button`): it owns its presses, its pressed, selected, busy and disabled states and its role for screen readers, and leaves its look to the pack: `Button.plate` and `Button.plate.<variant>` dress it, every state included. Its content is a row, centred.

`import { Button } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/button.tsx`).

| Prop | Type | What |
|---|---|---|
| `variant?` | `ButtonVariant \| undefined` | The look, rune-kit's matrix: `neutral` (the default), `primary` (the main action), `danger`, `warning`, `contrast` (the inverse of the page), `outline` (a rim, no fill) and the ghosts `ghost`, `ghost-muted` and `ghost-faint` (no plate until hovered). `quiet` is `ghost-muted`. `menu` is a front-end menu entry (a main or pause menu's row: type until it is the one, then filled in the accent) and `menu-danger` one that ends something (Quit game). The pack dresses each through `Button.plate.<variant>`. |
| `size?` | `"sm" \| "md" \| "lg" \| undefined` | Its padding and type: `sm`, `md` (the default) or `lg`. Each is a touch target 44 px tall or more. |
| `small?` | `boolean \| undefined` | `size="sm"`, its old name. |
| `pressed?` | `boolean \| undefined` | A toggle that is on: screen readers say so, and `aria-pressed:` styles apply. |
| `selected?` | `boolean \| undefined` | The current one of a set (a tab, a page): screen readers say so, and `selected:` styles apply. |
| `busy?` | `boolean \| undefined` | Its action is running: screen readers say so, it shows the pack's spinner (`Button.spinner`) and `aria-busy:` styles apply. Presses still count; set `disabled` as well to refuse them. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its content is not words (an icon). |
| `description?` | `string \| undefined` | What pressing it does, read after its name. |
| `role?` | `string \| undefined` | What it is for screen readers, when not a button (a tab, a menu item). |
| `name?` | `string \| undefined` | Its node name, which tests and the probe find it by (Resume). |
| `class?` | `string \| undefined` | Classes of the document that uses it; they come after its own, so they win. |
| `tabIndex?` | `number \| undefined` |  |
| `autofocus?` | `boolean \| undefined` | It takes the focus once it shows (a menu's first choice, a dialog's safe one). |
| `hold?` | `number \| undefined` | Seconds a press or confirm must be held before it presses (hold-to-confirm). |
| `onPress?` | `((e: UiEvent) => void) \| undefined` |  |
| `onFocus?` | `((e: UiEvent) => void) \| undefined` | It took the focus (a menu entry takes its list's bar). |
| `onPointerMove?` | `((e: UiEvent) => void) \| undefined` | The pointer moved over it. |
| `onHold?` | `((e: UiEvent) => void) \| undefined` | The hold's progress, 0 to 1, in `e.value` ("0" when let go early). |
| `onLongPress?` | `((e: UiEvent) => void) \| undefined` |  |
| `onAction?` | `Record<string, (e: UiEvent) => void> \| undefined` |  |
| `children?` | `any` |  |

Pack slots: `Button.plate.…`, `Button.spinner`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `primary`, `danger`, `warning`, `contrast`, `outline`, `ghost`, `ghost-muted`, `ghost-faint`, `large`, `pressed`, `busy`, `disabled`.

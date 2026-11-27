<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Popover"></a>
### `Popover`

Content by its trigger (NSH base's `Popover`: a filter, a quick detail): the trigger says whether it is expanded, and the panel follows the trigger's box wherever it moves (`anchor`). The pack draws the panel (`Popover.frame`), which pops in by its own `starting:`.

`import { Popover } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/popover.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Shown or not. The game owns it: a press on the trigger reports `onOpenChange` with the next value, `back` with false. |
| `placement?` | `"bottom" \| "top" \| undefined` | Where it stands by its trigger: `bottom` (the default) or `top`. |
| `label?` | `string \| undefined` | What screen readers call it. |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its panel. |
| `onOpenChange?` | `((e: UiEvent, open: boolean) => void) \| undefined` |  |
| `trigger?` | `any` | What opens it: a control its presses go to. |
| `children?` | `any` |  |

Pack slots: `Popover.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.

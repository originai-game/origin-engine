<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Collapsible"></a>
### `Collapsible`

A section whose content is shown or left out (NSH base's `Collapsible`): its header is a button that says whether it is expanded, and its content enters and leaves by the pack's `Collapsible.panel`. The pack draws the header (`Collapsible.trigger`) and the chevron that turns (`Collapsible.chevron`).

`import { Collapsible } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/collapsible.tsx`).

| Prop | Type | What |
|---|---|---|
| `open` | `boolean` | Its content is shown. The game owns it: a press on the header reports the next value with `onOpenChange`. |
| `title` | `string` | The header's text. |
| `disabled?` | `boolean \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it. |
| `onOpenChange?` | `((e: UiEvent, open: boolean) => void) \| undefined` |  |
| `children?` | `any` |  |

Pack slots: `Accordion.item`, `Collapsible.chevron`, `Collapsible.panel`, `Collapsible.trigger`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `open`, `closed`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Accordion"></a>
### `Accordion`

Stacked sections that expand and collapse (NSH base's `Accordion`), one at a time unless `multiple`, each a `Collapsible` in the pack's `Accordion.item`; the arrows move between their headers.

`import { Accordion } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/collapsible.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `readonly string[]` | The open sections' values. The game owns them: a press reports the next list with `onChange`. |
| `sections` | `readonly AccordionSection[]` |  |
| `multiple?` | `boolean \| undefined` | Several may be open at once; otherwise opening one closes the others. |
| `class?` | `string \| undefined` | Classes of the document that uses it. |
| `onChange?` | `((e: UiEvent, value: string[]) => void) \| undefined` |  |

Each `AccordionSection`:

| Field | Type | What |
|---|---|---|
| `value` | `string` |  |
| `title` | `string` |  |
| `disabled?` | `boolean \| undefined` |  |
| `content` | `() => any` | What it shows while open. |

Pack slots: `Accordion.item`, `Collapsible.chevron`, `Collapsible.panel`, `Collapsible.trigger`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `open`, `closed`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Select"></a>
### `Select`

One value picked from a list (NSH base's `Select`): a combobox for screen readers whose left/right arrows step the value while closed, and whose list opens anchored under the trigger, clamped inside the viewport and scrollable when long. The pack draws the trigger (`Select.trigger`, `aria-expanded:` while open), the list (`Select.list`) and each option (`Select.option`, `selected:` for the chosen one).

`import { Select } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/select.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `string` | The chosen option's value. The game owns it: a pick reports `onChange`. |
| `options` | `readonly SelectOption[]` |  |
| `open?` | `boolean \| undefined` | Its list is shown. The game owns it: a press on the trigger reports `onOpenChange`, a pick or `back` closes it. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What screen readers call it ("Quality"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its trigger. |
| `onChange?` | `((e: UiEvent, value: string) => void) \| undefined` |  |
| `onOpenChange?` | `((e: UiEvent, open: boolean) => void) \| undefined` |  |

Each `SelectOption`:

| Field | Type | What |
|---|---|---|
| `value` | `string` |  |
| `label` | `string` |  |
| `disabled?` | `boolean \| undefined` |  |

Pack slots: `Select.list`, `Select.option`, `Select.trigger`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `closed`, `open`.

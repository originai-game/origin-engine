<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-NumberInput"></a>
### `NumberInput`

A number stepped up and down (NSH base's `NumberInput`: a quantity to buy, a count to send): a spin button for screen readers whose arrows, Page Up and Down, Home and End step it, between a decrement and an increment button. The pack draws its box (`NumberInput.frame`), its figure (`NumberInput.field`) and its buttons (`NumberInput.step`).

`import { NumberInput } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/number-input.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The number. The game owns it: a step or a key reports the next one with `onChange`. |
| `min?` | `number \| undefined` | The bounds a step holds it to; none by default. |
| `max?` | `number \| undefined` |  |
| `step?` | `number \| undefined` | What one step moves it by; default 1. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it ("Quantity"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, value: number) => void) \| undefined` |  |

Pack slots: `NumberInput.field`, `NumberInput.frame`, `NumberInput.step`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `at min`, `disabled`.

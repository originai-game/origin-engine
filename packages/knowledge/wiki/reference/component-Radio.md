<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Radio"></a>
### `Radio`

One option of a set the player picks one of (NSH base's `Radio`), in a `RadioGroup`: it owns its presses and state for screen readers; a checked radio reports nothing, as it cannot uncheck itself. The pack draws its circle (`Radio.indicator`) and the dot (`Radio.dot`).

`import { Radio } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/radio.tsx`).

| Prop | Type | What |
|---|---|---|
| `checked?` | `boolean \| undefined` | Checked or not. The game owns the value: a press on an unchecked radio, or the arrows of its group reaching it, asks for `true` with `onChange`. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its children have no text that says it. |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, checked: true) => void) \| undefined` |  |
| `children?` | `any` | The label after the circle. |

Pack slots: `Radio.dot`, `Radio.indicator`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `unchecked`, `checked`, `disabled`.

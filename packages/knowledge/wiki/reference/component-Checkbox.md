<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Checkbox"></a>
### `Checkbox`

A control that is checked, unchecked or indeterminate (NSH base's `Checkbox`): it owns its presses, keys and state for screen readers; the pack draws its box (`Checkbox.indicator`, whose `checked:` and `indeterminate:` follow it) and the marks inside (`Checkbox.mark`, `Checkbox.dash`).

`import { Checkbox } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/checkbox.tsx`).

| Prop | Type | What |
|---|---|---|
| `checked?` | `boolean \| undefined` | Checked or not. The game owns the value: a press reports the next one with `onChange`, and the checkbox shows what the game passes back. |
| `indeterminate?` | `boolean \| undefined` | Neither (a "select all" over a partial selection), over `checked`: screen readers say mixed, and a press asks for checked. |
| `disabled?` | `boolean \| undefined` |  |
| `invalid?` | `boolean \| undefined` | Its value fails the game's checks: screen readers say invalid, and `aria-invalid:` styles apply. |
| `label?` | `string \| undefined` | What a screen reader calls it when its children have no text that says it. |
| `description?` | `string \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, checked: boolean) => void) \| undefined` | The value a press asks for: `true` from unchecked and from indeterminate, `false` from checked. |
| `children?` | `any` | The label after the box. |

Pack slots: `Checkbox.dash`, `Checkbox.indicator`, `Checkbox.mark`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `unchecked`, `checked`, `indeterminate`, `disabled`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-TextField"></a>
### `TextField`

A labelled single-line text field: the label (with a required mark), the pack's field box (`Field.frame`, every state), and under it the description, or the error while there is one, and a character count when it has a `maxLength`. The game owns the value.

`import { TextField } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/field.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | The field's visible label, also its name for screen readers. |
| `value?` | `string \| undefined` | The value; the game owns it and writes back each `onChange`. |
| `placeholder?` | `string \| undefined` |  |
| `description?` | `string \| undefined` | A line under the field explaining it; replaced by `error` while there is one. |
| `error?` | `string \| undefined` | What is wrong with the value; non-empty marks the field invalid. |
| `invalid?` | `boolean \| undefined` | Marks the field invalid without words (the error is shown elsewhere). |
| `required?` | `boolean \| undefined` | A mark after the label; the game still checks the value. |
| `maxLength?` | `number \| undefined` | The most characters it takes; shows a count of them under the field. |
| `type?` | `"text" \| "password" \| "number" \| "email" \| "url" \| "tel" \| "search" \| undefined` |  |
| `disabled?` | `boolean \| undefined` |  |
| `readOnly?` | `boolean \| undefined` |  |
| `autofocus?` | `boolean \| undefined` |  |
| `class?` | `string \| undefined` | Classes after the field group's own. |
| `onChange?` | `((e: UiEvent, value: string) => void) \| undefined` |  |
| `onSubmit?` | `((e: UiEvent) => void) \| undefined` | Enter pressed in a single-line field. |

Pack slots: `Field.area`, `Field.error`, `Field.frame`, `Field.hint`, `Field.label`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `described`, `error`, `disabled`.

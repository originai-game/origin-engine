<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Switch"></a>
### `Switch`

A setting that is on or off and takes effect at once (NSH base's `Switch`): it owns its presses, keys and state for screen readers; the pack draws its track (`Switch.track`) and the thumb that moves along it (`Switch.thumb`), whose `checked:` and `disabled:` follow the switch.

`import { Switch } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/switch.tsx`).

| Prop | Type | What |
|---|---|---|
| `checked?` | `boolean \| undefined` | On or off. The game owns the value: a press reports the opposite with `onChange`. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its children have no text that says it. |
| `description?` | `string \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, checked: boolean) => void) \| undefined` |  |
| `children?` | `any` | The label before the track. |

Pack slots: `Switch.thumb`, `Switch.track`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `off`, `on`, `disabled`.

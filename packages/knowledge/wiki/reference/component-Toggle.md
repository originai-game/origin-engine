<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Toggle"></a>
### `Toggle`

A button that is on or off (NSH base's `Toggle`), alone or in a `ToggleGroup`: it owns its presses and its pressed state for screen readers; the pack dresses it (`@Toggle.item`, `aria-pressed:` while on).

`import { Toggle } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/toggle.tsx`).

| Prop | Type | What |
|---|---|---|
| `pressed?` | `boolean \| undefined` | On or off. The game owns the value: a press reports the opposite with `onChange`. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its children have no text that says it (an icon). |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, pressed: boolean) => void) \| undefined` |  |
| `children?` | `any` |  |

Pack slots: `Toggle.item`, `ToggleGroup.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `segmented`.

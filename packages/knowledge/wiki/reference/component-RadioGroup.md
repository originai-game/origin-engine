<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RadioGroup"></a>
### `RadioGroup`

The set around `Radio`s (NSH base's `RadioGroup`): one stop of Tab, at the checked radio; the arrows move among its radios and confirm/press selects the focused radio without trapping gamepad navigation at the boundary. It holds no value: the game keeps which radio is checked.

`import { RadioGroup } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/radio.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string \| undefined` | What screen readers call the set ("Difficulty"). |
| `class?` | `string \| undefined` | Classes of the document that uses it: its layout (`flex-row` for a row). |
| `children?` | `any` |  |

Pack slots: `Radio.dot`, `Radio.indicator`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `unchecked`, `checked`, `disabled`.

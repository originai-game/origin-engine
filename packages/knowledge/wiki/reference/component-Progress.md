<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Progress"></a>
### `Progress`

A linear progress bar (NSH base's `Progress`): it owns its role, its range 0 to 100 and the indicator's width by the value; the pack draws its track (`Progress.track`) and its indicator (`Bar.fill`, which eases as the value moves).

`import { Progress } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/progress.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | How far the work is, as a percentage from 0 to 100 (clamped): NSH base's and gpui-base's range. |
| `indeterminate?` | `boolean \| undefined` | The extent is unknown (connecting, preparing): screen readers hear no value, and the indicator is the pack's `Progress.indeterminate`. |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `label?` | `string \| undefined` | What screen readers say is in progress. |

Pack slots: `Bar.fill`, `Progress.fill`, `Progress.indeterminate`, `Progress.track`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `empty`, `third`, `full`, `indeterminate`.

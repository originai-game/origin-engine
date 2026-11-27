<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Divider"></a>
### `Divider`

A hairline rule between groups, horizontal or vertical, optionally with words centred on it. The pack draws the line (`Divider.line`) and the words (`Divider.label`). Read only; screen readers hear a separator.

`import { Divider } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/divider.tsx`).

| Prop | Type | What |
|---|---|---|
| `orientation?` | `"horizontal" \| "vertical" \| undefined` | `horizontal` (the default) or `vertical`. |
| `label?` | `string \| undefined` | Words centred on a horizontal rule (OR, Rewards); empty: a plain rule. |
| `class?` | `string \| undefined` | Classes after its own (a margin). |

Pack slots: `Divider.label`, `Divider.line`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `plain`, `labelled`.

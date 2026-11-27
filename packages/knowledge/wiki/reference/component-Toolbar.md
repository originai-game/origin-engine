<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Toolbar"></a>
### `Toolbar`

A bar of controls (NSH base's `Toolbar`: a player's transport, a palette of tools): one stop of Tab, at the item the focus last left; the arrows move among its items without pressing them, a `ToggleGroup`'s inside it included (WAI-ARIA's toolbar). The pack draws its bar (`Toolbar.frame`).

`import { Toolbar } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/toolbar.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string \| undefined` | What screen readers call the toolbar ("Formatting"), when a screen has more than one. |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `children?` | `any` | Its `Button`s, `Toggle`s, `ToggleGroup`s and `ToolbarSeparator`s. |

Pack slots: `Toolbar.frame`, `Toolbar.separator`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.

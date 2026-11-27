<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Tabs"></a>
### `Tabs`

The tab list around `Tab`s (NSH base's `Tabs`): one stop of Tab, at the selected tab; the arrows move among its tabs and select each one they reach (WAI-ARIA's tab list, automatic activation). It holds no value and no content: the game shows the selected tab's content beside it.

`import { Tabs } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/tabs.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string \| undefined` | What screen readers call the list ("Sections"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `children?` | `any` |  |

Pack slots: `Tab.count`, `Tab.indicator`, `Tab.item`, `Tab.list`, `Tab.mark`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.

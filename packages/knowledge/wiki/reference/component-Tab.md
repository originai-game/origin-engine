<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Tab"></a>
### `Tab`

One tab of a tab list (NSH base's `Tab`), in `Tabs` with the others: it owns its presses and its selected state for screen readers; the pack dresses it (`Tab.item`) and the mark under the selected one (`Tab.mark`), which slides from the tab it replaces (`Tab.indicator`). A count beside its label is a `Text` of `packClass("Tab", "count")`, lit while selected.

`import { Tab } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/tabs.tsx`).

| Prop | Type | What |
|---|---|---|
| `selected?` | `boolean \| undefined` | The current tab of its list: screen readers say so, `selected:` styles apply and the list's mark sits under it. The game owns the value. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its children have no text that says it (an icon). |
| `description?` | `string \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onPress?` | `((e: UiEvent) => void) \| undefined` | It was pressed, or the arrows of its list reached it: the game selects it. |
| `children?` | `any` |  |

Pack slots: `Tab.count`, `Tab.indicator`, `Tab.item`, `Tab.list`, `Tab.mark`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Panel"></a>
### `Panel`

A titled panel: a window's frame (`Panel.frame`), the pack's room and gaps inside it (`Panel.body`) and its heading (`Panel.title`, the words or the band they sit on). Its content is a column. Every look is the pack's.

`import { Panel } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/panel.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string \| undefined` | Its heading (Settings, Inventory); empty: none. |
| `class?` | `string \| undefined` | Classes of the document that uses it (a width, a placement), after its own. |
| `children?` | `any` | Its content, laid out in a column. |

Pack slots: `Panel.body`, `Panel.frame`, `Panel.title`.

Fixtures: `default`.

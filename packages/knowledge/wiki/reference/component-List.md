<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-List"></a>
### `List`

A long list that makes only the rows near its view (NSH's `List`): a scroll view as tall as every row, whose rows in view, and `overscan` more each way, exist and stand at their place; as it scrolls, rows that leave the view go and those that come in are made. Its scroll view draws its own bar (`Scrollbar.*`), and the keys scroll a focused row into view as anywhere.

`import { List } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/list.tsx`).

Pack slots: `Row.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `thousands`.

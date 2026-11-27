<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Breadcrumbs"></a>
### `Breadcrumbs`

A trail in type: dim links, faint chevrons, the current page bold on a lit underline. The trail wraps between crumbs, and a long current page takes a line of its own and wraps between its words. Reports choose (a crumb's key).

`import { Breadcrumbs } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/breadcrumbs.tsx`).

| Prop | Type | What |
|---|---|---|
| `items` | `ReadonlyArray<{ id: string; label: string; icon: string; current: boolean }>` | The crumbs, the current page last: each { id, label, icon, current }, current true on the last. |
| `label` | `string` | What a screen reader calls it; empty is Breadcrumb. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `long`.

Actions: `choose` (Go back).

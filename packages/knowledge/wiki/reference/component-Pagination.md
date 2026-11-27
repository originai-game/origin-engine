<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Pagination"></a>
### `Pagination`

A row of cut keys, the current page lit.

`import { Pagination } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/pagination.tsx`).

| Prop | Type | What |
|---|---|---|
| `pages` | `ReadonlyArray<{ id: string; label: string; current?: boolean; gap?: boolean }>` | The page buttons, a keyed collection of { id, label, current }; a gap is { id, label: "…", gap: true }. |
| `label` | `string` | The navigation's accessible name (Pages). |
| `atStart?` | `boolean` | The first page is shown: Previous takes no press. |
| `atEnd?` | `boolean` | The last page is shown: Next takes no press. |
| `previousLabel?` | `string \| undefined` | Previous's accessible name (Previous page); absent: the navigation's label. |
| `nextLabel?` | `string \| undefined` | Next's accessible name (Next page); absent: the navigation's label. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPage?` | `(e: UiEvent, key?: string) => void` | The page event, with the row's key where it comes from a row. |
| `onPrevious?` | `(e: UiEvent, key?: string) => void` | The previous event, with the row's key where it comes from a row. |

Fixtures: `default`.

Actions: `previous` (Previous page), `next` (Next page), `page` (Go to page).

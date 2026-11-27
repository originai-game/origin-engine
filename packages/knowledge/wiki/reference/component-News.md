<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-News"></a>
### `News`

News and updates: articles with their date, kind, picture and summary, one expanded at a time with its body and action. Loading and an error each have their state; the game owns the articles.

`import { News } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/news.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses News and updates. |
| `entries?` | `ReadonlyArray<{ id: string; title: string; date: string; kind: string; summary: string; body: string; image: Texture \| 0; unread: boolean; pinned: boolean; actionLabel: string }>` | Keyed { id, title, date, kind, summary, body, image, unread, pinned, actionLabel } articles in the game's order. kind is game-formatted text (Patch notes, Event); body is plain multiline text, never HTML. image is a loaded texture handle or 0. Default []. |
| `selected?` | `string` | Expanded article id, game-owned. A toggle reports choose with that key; the game may clear it to collapse. |
| `loading?` | `boolean` | Loading articles; no article actions are present. Default false. |
| `disabled?` | `boolean` | Disable article toggles and actions. Default false. |
| `error?` | `string` | Failure explanation with retry; empty is no error. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `overflow`.

Actions: `choose` (Toggle an article by stable key), `open` (Take the selected article's optional action by stable key), `retry` (Retry loading articles).

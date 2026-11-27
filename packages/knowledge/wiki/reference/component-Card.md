<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Card"></a>
### `Card`

A card: a picture across its top, a heading, a line or two under it, a badge at its top end and the actions the caller supplies. It reports onOpen.

`import { Card } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/card.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | Its heading. |
| `body` | `string` | A line or two under the heading; empty: none. |
| `image` | `Texture \| 0` | A picture the game registered across its top; 0: none. |
| `badge` | `string` | A badge at its top end (New, -20%); empty: none. |
| `actions?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the actions snippet). |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `open` (Open).

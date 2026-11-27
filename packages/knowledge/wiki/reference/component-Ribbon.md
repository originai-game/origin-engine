<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Ribbon"></a>
### `Ribbon`

A strip across a corner of whatever it lies over (or a stamp across its middle). Its box only holds and cuts the strip: it draws nothing of its own, so the card under it shows. A game puts the box in the corner it names of a positioned card that clips (`overflow: hidden`), and the card's own corner cuts the strip's ends.

`import { Ribbon } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/ribbon.tsx`).

| Prop | Type | What |
|---|---|---|
| `text` | `string` | The ribbon's words (NEW, SALE). |
| `corner` | `string` | Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp). |
| `tone` | `string` | Its tone: primary (empty), success, warn, danger or info. |

Fixtures: `default`, `topLeft`, `bottomRight`, `bottomLeft`, `stamp`, `overflow`.

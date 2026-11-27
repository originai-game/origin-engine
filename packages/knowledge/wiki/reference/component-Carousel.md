<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Carousel"></a>
### `Carousel`

Pages of art on the HUD plate: the current one ringed in the accent, its neighbours set back; the place is a row of leaning pips between cut arrow keys. Reports prev and next.

The strip scrolls without snapping to a page; the lit pip follows the index.

`import { Carousel } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/carousel.tsx`).

| Prop | Type | What |
|---|---|---|
| `pages` | `ReadonlyArray<{ id: string; title: string; body: string; image: Texture \| 0 }>` | The pages: each { id, title, body, image }, image a texture handle (0: none). |
| `index` | `number` | The current page, from 0 (its dot lit). |
| `label` | `string` | What a screen reader calls it. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPrev?` | `(e: UiEvent, key?: string) => void` | The prev event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`, `empty`, `first`.

Actions: `prev` (Previous), `next` (Next).

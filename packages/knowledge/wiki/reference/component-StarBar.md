<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StarBar"></a>
### `StarBar`

A score against its thresholds: the figure over a track that fills in the style's gold, each threshold a star set on the track where it falls, lit once the score reaches it. The stars are the style's rating stars, the same as a level's or a results screen's. The style draws SVG stars; here each is the atlas star icon in a seat, which keeps their shape and colour but not the style's glow.

`import { StarBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/star-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value?` | `number` | The score. |
| `max?` | `number` | The bar's full score (its last threshold). Default 1. |
| `stars?` | `ReadonlyArray<StarBarStar>` | The thresholds: lit once the score reaches each. |
| `label?` | `string` | A caption over the score; empty is none. |
| `valueText?` | `string` | The score in words instead of the grouped number. |
| `compact?` | `boolean` | One row: the score beside a shorter bar. |

Each `StarBarStar`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `at` | `number` | Where it sits on the track, a fraction of max (0 to 1). |

Pack slots: `Plate.frame`.

Fixtures: `default`.

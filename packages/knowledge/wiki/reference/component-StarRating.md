<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StarRating"></a>
### `StarRating`

A row of rating stars, each lit once it is earned. The style draws the stars as SVG glyphs (an outlined empty star, a gold one with a lit edge and a glow); the engine draws no vector shapes, so each is the atlas star icon, one tint lit and a dim one unearned, landing with the pop entrance (the style's land, without its overshoot) delayed by the star's index.

`import { StarRating } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/star-rating.tsx`).

| Prop | Type | What |
|---|---|---|
| `stars` | `ReadonlyArray<{ id: string; index: number; lit: boolean }>` | A keyed collection of { id, index (0 first), lit }: one star each; a lit star is earned and drawn filled. |
| `label` | `string` | What it says to a screen reader (2 of 3 stars). |
| `size?` | `"md" \| "lg"` | md (the default): 32 px stars in a line. lg: a result's grade, 78 px stars (52 on a phone) with the middle of three raised and larger, as the puzzle summary draws them. |
| `class?` | `string \| undefined` | Classes after the row's own (a placement, a margin). |

Fixtures: `default`, `none`, `large`, `all`.

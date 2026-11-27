<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyResources"></a>
### `StrategyResources`

The resource bar at the top left: each stockpile's picture in a cut tile, its amount large (a ceiling after it, dim, as the population's), and its income under it, green while it grows and red while it drains, or the game's word (Near cap) in gold at a limit; the age or tier on a lit tag. The game owns every figure, rate and limit; read only.

`import { StrategyResources } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/resources.tsx`).

| Prop | Type | What |
|---|---|---|
| `resources?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; amount: string; cap: string; rate: string; trend: string; note: string; warn: boolean }>` | The stockpiles in the game's order, keyed rows of { id, name, icon, glyph, amount, cap, rate, trend, note, warn }: name for a reader (Gold); icon its picture (og.load.texture), 0 draws glyph (an icon of the atlas) instead; amount the store as the game words it (1,250); cap a ceiling shown after it (/ 50, the population's), empty none; rate the income as the game words it (+12/s), under the amount, inked by trend (gain, loss or none: data-trend); note a word in place of the rate (Near cap), empty none; warn marks a store at its limit in gold (data-warn). |
| `tier` | `string` | The age or tier as the game words it (Castle Age), on a lit tag after the stockpiles; empty hides it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `tier`, `capped`, `empty`, `overflow`.

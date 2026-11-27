<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgHits"></a>
### `RpgHits`

Damage and healing figures beside the fight: a critical hit's figure largest, gold and slanted under its call, older figures stepping down and away, a burn in an ember ink with its word, a heal in green. The game owns every figure and where the page puts them; read only.

`import { RpgHits } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/hits.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<{ id: string; amount: number; kind: string; call: string; word: string }>` | The latest damage and healing figures, newest first, keyed rows of { id, amount, kind, call, word }: kind normal, critical (gold and slanted, the largest), burn (damage over time in an ember ink, its word beside it), heal (green with a plus) or miss (on data-kind); call the words over a critical as the game words them (Critical · Weak point), empty for none; word the figure's kind as the game words it (burn), empty for none. The newest stands largest; when there are older ones it keeps room before it, where they step down and away (a phone shows the newest alone); the game trims the list and places it. |

Fixtures: `default`, `heal`, `miss`, `overflow`.

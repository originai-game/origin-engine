<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmChain"></a>
### `RhythmChain`

The score chain: a lit plate with the multiplier, and beside it the steps to the next one as leaning pips. The game counts both; read only.

`import { RhythmChain } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/chain.tsx`).

| Prop | Type | What |
|---|---|---|
| `multiplier` | `string` | The score multiplier as the game words it (×4); empty hides the chain. |
| `step` | `number` | How many of the steps to the next multiplier are lit. |
| `steps?` | `number` | The steps to the next multiplier; 0 hides the strip (at its top multiplier). Default 8. |
| `hit` | `number` | Changes when the multiplier rises: the plate pops again. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `top`, `overflow`.

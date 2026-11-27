<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterRound"></a>
### `ShooterRound`

The round on a plate lit along its top, its time left beside it, and the score with the player's side in its blue against the other's red. The game owns the clock and the score; read only. The plate wears the pack's cut and keeps no rim or urgent glow (a cut takes one fill).

`import { ShooterRound } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/round.tsx`).

| Prop | Type | What |
|---|---|---|
| `round?` | `number` | The round in play; 0 hides the word. |
| `clock?` | `string` | The round's time left as the game words it (1:42); empty hides it. |
| `ours?` | `number` | The player's side's score. |
| `theirs?` | `number` | The other side's score; negative hides the score. |
| `urgent?` | `boolean` | The game says time is short: the plate's lit edge and the clock turn to danger. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `urgent`, `clock`, `overflow`.

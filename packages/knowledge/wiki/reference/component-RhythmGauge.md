<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmGauge"></a>
### `RhythmGauge`

A tall gauge at a screen edge, cut into ten cells and leaning toward the play field: the run's life (green; danger red with a pulsing rim under the game's mark, Danger and Failed in words) or its fever charge (pink; Ready and the key that spends it). Its height is the space the page gives it. The game owns the value and every verdict; read only.

`import { RhythmGauge } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/gauge.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind?` | `string` | What it measures: life (the run fails at empty) or fever (a charge the player spends when full); on data-kind. Default "life". |
| `value` | `number` | How full, 0 to 1. |
| `side?` | `string` | The screen edge it stands at, start or end: it leans toward the play field. Default "start". |
| `danger` | `boolean` | The game says life is low: the gauge turns to danger, its rim pulses and Danger shows under it. |
| `failed` | `boolean` | The game says the run is failed (life gone): Failed shows under it. |
| `ready` | `boolean` | The game says the charge is ready to spend: the gauge glows and Ready shows. |
| `key` | `string` | The input that spends it as the game names it (Space, RB); empty hides the key cap. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `fever`, `ready`, `danger`, `failed`, `overflow`.

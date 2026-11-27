<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingBoost"></a>
### `RacingBoost`

A boost meter (nitro, ERS) beside the tachometer: its picture in a cut tile, the name with the key that fires it, and a leaning bar cut into cells, glowing while it fires. The game owns the store and when it fires; read only.

`import { RacingBoost } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/boost.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The meter's name as the game words it (Nitro, ERS, Boost); empty takes the kit's (Boost). |
| `icon` | `Texture \| 0` | The meter's picture (og.load.texture) in a cut tile before it; 0 hides it. |
| `key` | `string` | The input that fires it as the game names it (Shift), on a key cap after the label; empty for none; hidden under a touch screen. |
| `value` | `number` | What is left now. |
| `max?` | `number` | The most there can be. Default 1. |
| `cells?` | `number` | The cells the bar is cut into (5); 1 draws one bar. Default 5. |
| `firing` | `boolean` | It is firing now: the fill glows (data-firing). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `firing`, `empty`, `overflow`.

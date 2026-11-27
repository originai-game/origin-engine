<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyClock"></a>
### `StrategyClock`

The match clock at the top right: what it counts (Day 3), the time large and the game's speed on a tag, or Paused. The game owns the time and the speed; read only.

`import { StrategyClock } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/clock.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | What the clock counts as the game words it (Day 3, Match), before the time; empty hides it. |
| `time` | `string` | The time as the game words it (12:34). |
| `speed` | `string` | The game's speed as it words it (×1, ×2), on a tag after the time; empty hides it. |
| `paused` | `boolean` | The game is paused: the Paused word stands on the tag in place of the speed (data-paused). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `paused`, `empty`, `overflow`.

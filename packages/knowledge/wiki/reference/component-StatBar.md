<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StatBar"></a>
### `StatBar`

A stat with its bar: the icon and label before a well that fills to value in its kind's colour, the heal and recoverable shares past it, a trailing ghost behind losses, the shield along its top, a notch at each interior block boundary, and the figure at its end (inside the track on valueInside). Health turns warn under low and danger under critical. Read only.

`import { StatBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/stat-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value. |
| `max?` | `number` | Its maximum. Default 1. |
| `kind?` | `string` | What it measures (its colour): health, mana, stamina or xp; health turns warn and danger as it falls. Default health. |
| `segment?` | `number` | Value per block: a notch at each block's end (0: one bar). |
| `shield?` | `number` | Shield or armour on top of the value, a strip over the bar. |
| `shieldMax?` | `number` | The most shield; 0 hides the strip. |
| `heal?` | `number` | Healing on its way, a pale preview past the fill. |
| `recoverable?` | `number` | Damage that can still come back, drawn grey between the value and this. |
| `ghost?` | `number` | Value before the latest loss, held as a trailing ghost behind the fill. Absent, the ghost rests at the fill and lags it on each loss. |
| `label?` | `string` | A short name at its start (HP); empty is none. |
| `icon?` | `string` | An icon of the atlas before it; empty is none. |
| `showValue?` | `boolean` | The value as text at its end. |
| `valueInside?` | `boolean` | The value inside the track at its end, so the bar keeps its length. |
| `valueText?` | `string` | The value as text instead of value / max. |
| `low?` | `number` | The fraction under which it is low (0.3). Default 0.3. |
| `critical?` | `number` | The fraction under which it is critical (0.15). Default 0.15. |
| `bare?` | `boolean` | No HUD plate behind it. |
| `name?` | `string` | What a screen reader calls it; empty is the label, else Health. |

Pack slots: `Bar.fill`, `Bar.frame`, `Bar.frame.low`, `Bar.ghost`, `Bar.tick`, `Bar.trail`, `Plate.frame`.

Fixtures: `default`.

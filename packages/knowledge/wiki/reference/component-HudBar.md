<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudBar"></a>
### `HudBar`

One bar of a HUD, a block a plate is built from: it fills to value over max in its kind's colour over a dark well with a lit rim; a loss leaves a warm ghost that drains after the fill, a low bar turns to danger, segment cuts it into cells, and showValue stands its figure over it. It is as wide as its place; a plate sizes the place. Read only. Its well, fill colours, ghost, notches and figure are the Bar component's pack slots, so a theme restyles every HUD bar at once.

`import { HudBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value now. |
| `max?` | `number` | Its most; 0 draws the bar empty. Default 1. |
| `kind?` | `string` | What it measures, which is its fill: health, mana, stamina, shield or xp. Default health. |
| `ghost?` | `number` | The value before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none). |
| `segment?` | `number` | Value per cell (one dodge's cost): the bar is cut into cells; 0 draws it whole. |
| `low?` | `boolean` | Low: the fill turns to danger and the rim glows. |
| `showValue?` | `boolean` | Its figure, value over max, upright over the bar; for a bar thick enough to hold a line of text. |
| `name?` | `string` | What a screen reader calls it; empty is its kind (Health, Mana, Stamina, Experience). |

Pack slots: `Bar.fill`, `Bar.frame`, `Bar.frame.low`, `Bar.frame.thick`, `Bar.ghost`, `Bar.tick`, `Bar.tone`, `Bar.tone.low`, `Bar.tone.…`, `Bar.trail`, `Bar.value`.

Fixtures: `default`, `mana`, `cells`, `shield`, `figure`, `low`, `empty`, `overflow`.

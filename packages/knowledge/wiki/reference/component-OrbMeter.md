<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-OrbMeter"></a>
### `OrbMeter`

A glass vessel in a bezel, over the scene: a bezel with its light and shade, a socket, a dark well in its liquid's hue with the liquid standing to the value, a surface line, a gleam blob riding the surface and the figure in the middle. The style's SVG waves, glare arcs and ornament have no TSX surface: the liquid is a flat fill with a surface line, the glare a white blob, the ornament dropped (the default pack draws none). The packs' --orb-meter-* tokens are resolved to the dark scheme's values here.

`import { OrbMeter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/orb-meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value. |
| `max?` | `number` | Its maximum. Default 1. |
| `kind?` | `string` | Its liquid's colour: health or mana. |
| `showValue?` | `boolean` | The value in the middle. |
| `label?` | `string` | A caption under it; empty is none. |
| `name?` | `string` | What a screen reader calls it; empty is the label, else Health. |

Pack slots: `Orb.box`, `Orb.fill`, `Orb.fill.mana`, `Orb.glass`, `Orb.label`, `Orb.rim`, `Orb.value`, `Orb.well`, `Orb.well.mana`.

Fixtures: `default`, `mana`, `empty`, `full`.

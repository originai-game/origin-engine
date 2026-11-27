<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingTrackMap"></a>
### `RacingTrackMap`

The track map at the top right: the circuit as a light line over a dark band, the lap driven so far lit in the HUD's accent, the start line, the other cars as dots in their colours and the player's car as an arrow over them; the track's name and length under it. The game gives the shape and every position in the map's box; read only.

`import { RacingTrackMap } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/track-map.tsx`).

| Prop | Type | What |
|---|---|---|
| `path` | `string` | The track's racing line as an SVG path (d) in the map's 260 by 220 box; empty draws no road. |
| `driven` | `string` | The part of the lap the player has driven, as an SVG path in the same box, lit in the HUD's accent; empty draws none. |
| `startX` | `number` | The start line's place in the box, across. |
| `startY` | `number` | The start line's place in the box, down. |
| `startAngle` | `number` | The start line's turn in degrees. |
| `cars?` | `ReadonlyArray<{ id: string; x: number; y: number; heading: number; color: string; player: boolean; name: string }>` | The cars on the map, keyed rows of { id, x, y, heading, color, player, name }: x and y in the map's box; color the car's colour as a CSS colour (empty: neutral); player marks the player's car, drawn as an arrow turned to heading (degrees clockwise from up) over the others' dots; name for a reader. |
| `name` | `string` | The track's name as the game words it, under the map; empty hides it. |
| `length` | `string` | The track's length as the game words it (4.8 km), on a tag beside the name; empty hides it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `start`, `overflow`.

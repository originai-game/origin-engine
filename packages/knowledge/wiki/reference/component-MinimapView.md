<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-MinimapView"></a>
### `MinimapView`

A minimap: the map texture zoomed round the player's point, player-up or north-up, round or square, with its markers and the zone's name under it. It reports onPick for a tap on the glass.

`import { MinimapView } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/minimap-view.tsx`).

| Prop | Type | What |
|---|---|---|
| `texture?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the well with a grid. |
| `centerX?` | `number` | The map point at its centre (the player's), 0 to 1 across. |
| `centerY?` | `number` | The map point at its centre, 0 to 1 down. |
| `zoom?` | `number` | How much of the map it shows across, 0 to 1 (0.3). Default 0.3. |
| `heading?` | `number` | The player's heading in degrees clockwise from north. |
| `rotate?` | `boolean` | Player-up: the map turns with the heading; false keeps north up and turns the player's arrow. |
| `square?` | `boolean` | A square frame; false is round. |
| `markers?` | `ReadonlyArray<MinimapViewMarker>` | Its markers. |
| `zone?` | `string` | The zone's name under it; empty is none. |
| `onPick?` | `(e: UiEvent) => void` | Pick: a tap on the glass. |

Each `MinimapViewMarker`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `kind?` | `string` | ally, enemy, objective, ping, loot or poi. |
| `x` | `number` | On the map, 0 to 1 across. |
| `y` | `number` | On the map, 0 to 1 down. |
| `label?` | `string` | For a reader. |
| `pinned?` | `boolean` | Out of view, it stays pinned to the rim, dimmed, with a tick pointing its way (an objective, a party member); false drops it. |
| `elevation?` | `string` | above or below the player (another floor): a small chevron on the pip; empty is level. |

Pack slots: `Plate.frame`.

Fixtures: `pinned`, `default`.

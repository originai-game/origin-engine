<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterRadar"></a>
### `ShooterRadar`

A round radar: the map turned so north stays up around the player's point, the player's arrow by its heading and the blips the game hands it, clipped at the edge. Read only.

`import { ShooterRadar } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/radar.tsx`).

| Prop | Type | What |
|---|---|---|
| `map?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the dark well alone. |
| `centerX?` | `number` | The map point under the player, 0 to 1 across. |
| `centerY?` | `number` | The map point under the player, 0 to 1 down. |
| `zoom?` | `number` | How much of the map the radar shows across, 0 to 1 (0.4). |
| `heading?` | `number` | The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. |
| `blips?` | `ReadonlyArray<RadarBlip>` | What the radar shows. A blip off the radar's edge is clipped unless it is pinned. Read only. |

Each `RadarBlip`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `kind?` | `string` | ally, foe or objective (each its own shape and colour). |
| `x` | `number` | On the map, 0 to 1 across. |
| `y` | `number` | On the map, 0 to 1 down. |
| `label?` | `string` | Names it for a reader. |
| `pinned?` | `boolean` | Off the radar, it stays pinned on the inner rim, dimmed (the objective, a squadmate); false clips it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`, `edge`.

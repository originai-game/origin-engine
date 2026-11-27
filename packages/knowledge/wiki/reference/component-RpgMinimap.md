<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgMinimap"></a>
### `RpgMinimap`

The minimap at the screen's top corner: the map in a dark round well with a lit ring, the party, the foes and the objective as blips, the player's arrow at the centre turned to the heading and N on the ring's top; the place under it with the time of day on a tag. Read only.

`import { RpgMinimap } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the dark well alone. |
| `centerX?` | `number` | The map point under the player, 0 to 1 across. Default 0.5. |
| `centerY?` | `number` | The map point under the player, 0 to 1 down. Default 0.5. |
| `zoom?` | `number` | How much of the map the minimap shows across, 0 to 1 (0.5). Default 0.5. |
| `heading` | `number` | The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. |
| `blips?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string; pinned?: boolean }>` | What the minimap shows, keyed rows of { id, kind, x, y, label }: kind ally, foe, boss or objective (on data-kind, each its own shape and colour); x and y on the map, 0 to 1; label names it for a reader; pinned keeps it on the rim, dimmed, when it is out of range. A blip off the minimap's edge is otherwise clipped. |
| `place` | `string` | Where the player is as the game words it (Ashen Vale), under the map; empty hides it. |
| `clock` | `string` | The world's time of day as the game words it (06:40), on a tag beside the place; empty hides it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`, `edge`, `overflow`.

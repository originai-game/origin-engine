<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterPickups"></a>
### `ShooterPickups`

The pickup feed lower on the screen's side: each row an icon, the item's name and the count gained on a dark glass that fades toward the middle, its edge at the screen's side tinted by rarity. Read only. A row the For removes fades out on exit: by itself; the rows' FLIP reflow (the svelte's animate:flip) has no TSX surface and is dropped, as is the row's right-to-left mirror.

`import { ShooterPickups } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/pickups.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<PickupItem>` | What the player just picked up, newest first. The game trims old rows (a phone shows the newest); a new row slides in from the screen's edge and an old one fades out. Read only. |

Each `PickupItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What the game calls it. |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `quantity?` | `number` | The count gained (+30). |
| `rarity?` | `string` | common, rare, epic or legendary: the edge's and the count's colour. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `epic`, `overflow`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterEffects"></a>
### `ShooterEffects`

The effects on the player, over the vitals: each a small tile with two corners cut, its edge green for a boon and red for a harm, the time already run as a dark sweep round its art and its time or stacks on a tag under it. The game owns every timer; read only. The tiles' cut corners have no TSX surface: they are rounded ones.

`import { ShooterEffects } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/effects.tsx`).

| Prop | Type | What |
|---|---|---|
| `effects?` | `ReadonlyArray<ShooterEffect>` | The effects on the player in the game's order. Read only. |

Each `ShooterEffect`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls it (Haste). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `remaining?` | `number` | The share of its time still to run, 0 to 1: a dark sweep over the art; negative is no timer. |
| `label?` | `string` | Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. |
| `harmful?` | `boolean` | A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `untimed`, `overflow`.

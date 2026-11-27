<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudEffect"></a>
### `HudEffect`

One buff or debuff of a HUD, a block an effects row is built from: a small tile whose edge is its fill (green for a boon, red for a harm), its art inset inside it, the time already run darkened round the art and its time or stacks on a tag under it. The game owns every timer; read only. The tile's cut corner has no TSX surface: it is a rounded one.

`import { HudEffect } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/effect.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | What a reader calls it (Haste). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `remaining?` | `number` | The share of its time still to run, 0 to 1: a dark radial sweep over the art; negative is no timer. |
| `label?` | `string` | Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. |
| `harmful?` | `boolean` | A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `harmful`, `untimed`, `empty`, `overflow`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudPortrait"></a>
### `HudPortrait`

A portrait of a HUD, a block a plate is built from: the art (or the person glyph) inset in a lit edge. Its side is the HUD portrait size; a plate sets it on the place it gives the portrait. Read only. The style's octagon and its greying filter have no TSX surface: the edge is a rounded one and the dimmed art shows dimmed.

`import { HudPortrait } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/portrait.tsx`).

| Prop | Type | What |
|---|---|---|
| `portrait?` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. |
| `dimmed?` | `boolean` | Out of the fight: the art greyed. |
| `class?` | `string` | Extra classes for the root, to size it (NSH style). |

Fixtures: `default`, `dimmed`.

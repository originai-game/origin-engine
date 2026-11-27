<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterSquad"></a>
### `ShooterSquad`

The squad at the screen's top corner: each squadmate's portrait in an octagon, the name and a state in words, a thin shield bar cut into cells over a thin health bar with its ghost. The game owns every value and state; read only. Approximations: the portrait's octagon cut is rounded, the out state's grey-out is opacity, the cells' mask is hairline dividers, and a phone's side-by-side reflow keeps the plate's stack.

`import { ShooterSquad } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/squad.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<SquadMember>` | The player's squadmates in the game's order. The game owns every value and state; read only. |
| `segment?` | `number` | Shield per cell of each shield bar (25). |

Each `SquadMember`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | Their portrait, a texture (og.load.texture); 0 draws the person glyph. |
| `status?` | `string` | The state as the game words it (Reviving), empty for none. |
| `state?` | `string` | ok, down, reviving or out. |
| `shield?` | `number` | Shield now. |
| `shieldMax?` | `number` | Most shield; 0 hides the shield bar. |
| `health?` | `number` | Health now. |
| `healthMax?` | `number` | Most health. |
| `ghost?` | `number` | The health before the latest loss. |

Pack slots: `Bar.fill`, `Bar.trail`, `Plate.frame`.

Fixtures: `default`, `down`, `empty`, `overflow`.

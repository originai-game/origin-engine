<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Backdrop"></a>
### `Backdrop`

What a full-screen built-in stands on: the key art cropped to the screen, or, without art, the pack's designed ground (Screen.backdrop with its glow and vignette), never a flat colour; then the veil its words need. Over the running game (a pause menu) it draws the veil alone.

`import { Backdrop } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/backdrop.tsx`).

| Prop | Type | What |
|---|---|---|
| `art` | `Texture \| 0` | The key art (og.load.texture, or the project's art slot); 0: the pack's own ground (Screen.backdrop), lit and vignetted. |
| `veil?` | `"none" \| "side" \| "foot" \| "full" \| undefined` | The shade over it, for the words in front: `side` down the leading third (a main menu), `foot` along the foot (loading), `full` over all of it (pause, game over), `none`. |
| `over?` | `boolean \| undefined` | It stands over the running game, not a picture of its own: with no art, only the veil is drawn and the game shows through it. |
| `class?` | `string \| undefined` | Classes after its own. |

Pack slots: `Screen.backdrop`, `Screen.glow`, `Screen.veil`, `Screen.veil.foot`, `Screen.veil.side`, `Screen.vignette`.

Fixtures: `default`, `side`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HitFeedback"></a>
### `HitFeedback`

What a hit feels like on the HUD, opt in: the screen's edges flash as a hit lands, pulse while the game says the player is critical, and an arc points to where each recent hit came from. Every look and timing is the theme's (Hit.vignette, Hit.vignette.critical, Hit.ring, Hit.arc and the duration-hit and duration-arc tokens). It decides nothing: the game names the hits and when the player is critical. Read only.

`import { HitFeedback } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hit-feedback.tsx`).

| Prop | Type | What |
|---|---|---|
| `flash?` | `string` | The latest hit's id: each new id flashes the screen's edges once. Empty is none. |
| `critical?` | `boolean` | The player is critical, as the game decides: the edges pulse until it clears. Default false. |
| `hits?` | `ReadonlyArray<DamageDirectionHit>` | Recent hits, each drawn as an arc toward where it came from (DamageDirection). The game removes a hit when its mark should go. |

Pack slots: `Hit.vignette`, `Hit.vignette.critical`.

Fixtures: `default`, `critical`.

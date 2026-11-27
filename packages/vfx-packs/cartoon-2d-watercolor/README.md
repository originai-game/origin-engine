# Watercolour 2D effects

Install with `og vfx add cartoon-2d-watercolor`; spawn `base/vfx/cartoon-2d-watercolor/<name>` as an
og.Sprite flipbook (`og api show anim`). Watercolour effects for cartoon-2d's watercolor variant: light translucent-looking washes with no contour, a darker pigment rim instead.

Every effect is a `file.vfx2d` recipe drawn supersampled (`supersample: 4`):
the cook renders it at four canvas pixels an output pixel, outlines it and
shrinks it, so its edges are smooth and its sheet is filtered linearly.
An effect's pixels are the character's (96 to 160 px): draw the sprite at
1 world unit a pixel of the game's own scale, never resampled.

The same names, timing and pivots are in every cartoon pack
(`cartoon-2d`, `cartoon-2d-watercolor`, `cartoon-2d-storybook`,
`cartoon-2d-rubber-hose`), so a game changes look by changing the pack.
Each one-shot effect sends an og.FlipbookEvent `impact` (the `pixel-vfx`
guide). No gameplay, scoring, damage or UI logic; text is left to the
game's UI.

| Effect | Plays | Placement |
|---|---|---|
| hit | once | Where the hit lands; a struck enemy or wall. |
| hit-small | once | Where a light hit lands. |
| hit-heavy | once | Where a heavy hit lands; pair it with og.time.hitStop on its impact. |
| slash | once | At the swing's centre, +x forward; turn it with the sprite. |
| stomp | once | On the ground under a stomped enemy. |
| block | once | In front of a guarding character, +x toward the attack. |
| muzzle | once | At the barrel's mouth, +x forward. |
| bullet | loops | Loops on the moving shot; +x is where it flies. |
| bullet-hit | once | Where a shot ends. |
| charge | loops | Loops at the hand or barrel while a shot charges. |
| laser | loops | Loops from the emitter along +x; scale x for length. |
| fuse | loops | Loops at a lit fuse's tip. |
| explosion | once | On the ground where it goes off. |
| explosion-small | once | On the ground or a body where a small charge goes off. |
| poof | once | Where something appears or vanishes. |
| jump-dust | once | At the feet as a jump leaves the ground. |
| land-dust | once | At the feet on landing. |
| run-dust | loops | Loops behind running feet; +x is the way the runner faces. |
| dash | once | At the dashing body; +x is the way it dashes. |
| double-jump | once | At the feet of a second jump in the air. |
| skid | loops | Loops at the feet while skidding; +x is the way it slides. |
| splash | once | On the water line where something falls in. |
| splash-small | once | On the water line where something small drops in. |
| ripple | loops | Loops on the water line round a wading body or a float. |
| bubbles | loops | Loops under water at a diver or a vent. |
| coin | once | Where a coin is collected. |
| gem | once | Where a gem is collected. |
| heart | once | Where a life or health pickup is collected. |
| powerup | once | Round a character taking a power-up. |
| star-collect | once | Where a big collectible is taken. |
| chest-open | once | At an opened chest's lid line. |
| confetti | once | Where a win is celebrated; project a UI point into the scene. |
| confetti-rain | loops | Loops over a win screen's play area. |
| fireworks | once | In the sky over a win. |
| checkpoint | once | On the ground at a reached checkpoint. |
| heal | once | On the ground under the healed character. |
| buff | once | On the ground under a strengthened character. |
| debuff | once | Over a weakened character's head. |
| stun | loops | Loops over a stunned character's head. |
| poison | loops | Loops round a poisoned character. |
| burning | loops | Loops on a burning character; it stands on the ground. |
| frozen | once | On the ground under a frozen character. |
| shocked | once | Round a shocked character. |
| magic | once | At the hand or wand casting. |
| magic-bolt | loops | Loops on the moving spell; +x is where it flies. |
| magic-circle | loops | Loops on the ground under a caster. |
| portal | loops | Loops where a portal stands; its centre at the anchor. |
| teleport | once | On the ground where a character vanishes or arrives. |
| shield | loops | Loops round a shielded character's middle. |
| whirlwind | loops | Loops on the ground under a whirlwind. |
| campfire | loops | Loops on the ground at the logs. |
| torch | loops | Loops at a torch's head. |
| chimney-smoke | loops | Loops at a chimney's top. |
| leaves | loops | Loops over a play area under trees. |
| fireflies | loops | Loops in a dark play area. |
| twinkle | loops | Loops on something shiny: a gem, a star, a secret. |
| break-crate | once | At the centre of a broken crate. |
| break-rock | once | At the centre of a broken rock. |
| break-ice | once | At the centre of shattered ice or glass. |

All recipes are original MIT work, Origin Engine, written by
`scripts/vfx-cartoon.ts` for the Pixel VFX Forge renderer the cook runs
(`third_party/scenario-pixel-vfx`). The outline is an exact Euclidean
distance transform (Felzenszwalb and Huttenlocher, "Distance Transforms of
Sampled Functions", Theory of Computing 8, 2012; method only).

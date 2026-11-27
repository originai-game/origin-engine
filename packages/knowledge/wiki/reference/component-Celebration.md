<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Celebration"></a>
### `Celebration`

A reward over the HUD: confetti thrown from the top edge, a burst's ring and sparks, a level-up's ring of light with its word in gold, its new level and embers rising under it, or fireworks' five shells. Each play runs once per shot, with the pack's reward sound; a level-up's light (a column, a wash, rays and rising stars) is Flourish's, which sounds levelup. The particles are the engine's emitters drawn by `Particles` (./fx/*.particles.ts), one node per ink, so a pack's roles colour them; reduced motion draws none.

`import { Celebration } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/celebration.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind` | `string` | Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light behind the title and the new level, embers rising under them) or fireworks (shells bursting across its top). |
| `shots?` | `ReadonlyArray<{ id: string }>` | Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. |
| `title?` | `string` | levelUp: the big word; empty is Level up. |
| `value?` | `string` | levelUp: the new level (12); empty is none. |

Pack slots: `Celebrate.halo`, `Celebrate.title`.

Fixtures: `default`, `confetti`, `burst`, `fireworks`.

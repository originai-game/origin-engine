<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Flourish"></a>
### `Flourish`

A flourish around what it decorates, the engine's UI effect presets: a reward's flash, rays, ring and stars; a level-up's column of light, screen wash and rising stars; a purchase's fountain of spinning coins catching the light while the price pops; a rarity reveal that draws a charge in (glints gathering, a glow swelling, the target growing), then bursts (a flash, rays, a ring and shards, the target popping) and settles in lingering motes, each tier richer (rays from rare, double rays from epic, triple rays, a column and a screen wash from legendary); the beams draw behind the target, the rest over it; or a highlighted button's aura in the pack's primary colour, sweeping sheen and twinkling stars. Each play runs once per shot with its pack sound (highlight runs while it shows, silent). The particles are the engine's emitters (./fx/*.particles.ts) with the shared textures (`@styles/shared/fx/`), drawn by `Particles`, one node per ink, so a pack's colour roles colour them; reduced motion draws none.

`import { Flourish } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/flourish.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind` | `"reward" \| "levelUp" \| "purchase" \| "reveal" \| "collect" \| "highlight"` | Which one: reward (stars, rays and a ring of light thrown from it), levelUp (a column of light, a screen wash, stars rising and a ring), purchase (a fountain of spinning coins, the target popping), reveal (a charge drawn in, then a rarity's burst: richer from common to legendary), collect (coins flying from `from` into it, a counter, which flashes as they land) or highlight (an aura in the pack's primary colour pulsing, a sheen sweeping it and stars twinkling, while it shows). |
| `shots?` | `ReadonlyArray<{ id: string }>` | Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. highlight needs none. |
| `rarity?` | `string` | reveal: the rarity whose colour and tier it takes (common, uncommon, rare, epic, legendary, mythic); empty is legendary. |
| `from?` | `readonly [number, number]` | collect: where the coins start, px from its centre ([x, y], y down); default [0, 240], below it. |
| `icon?` | `string` | collect: the icon that flies (a currency); default coin. |
| `children?` | `any` | What it decorates (an icon, a slot, a price, a button); the effect is drawn over and around it. |
| `class?` | `string \| undefined` | Classes after its own: it hugs what it decorates; `w-full` (or a grid cell's stretch) lets a full-width target fill its place. |

Fixtures: `reward`, `levelUp`, `purchase`, `reveal`, `collect`, `revealLegendary`, `highlight`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PassTrack"></a>
### `PassTrack`

A season pass: a kicker with the season and the time left over the title, a gold Get premium while premium is not owned, the level on a hexagon beside a leaning bar to the next tier, and two lanes of tier tiles, premium (in gold) over free. Each lane scrolls sideways on its own; on a narrow screen its name sits above it. A tile says its tier, shows the reward and how much, and ends in Claim when it is ready and owned, Claimed once taken, or Locked (a premium tile always reads Locked without premium). The game owns progress, entitlement and every reward's state; the track reports claim (a reward's key), upgrade and retry.

`import { PassTrack } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/pass-track.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Season pass. |
| `season?` | `string` | The game's season name, a kicker over the title. |
| `refresh?` | `string` | The game's already formatted time remaining, after the season in the kicker; no clock runs here. |
| `level?` | `string` | The player's pass level as the game words it, on a hexagon beside the progress; empty hides it. |
| `progress?` | `number` | Progress to the next tier, 0 to 1, supplied by the game. Default 0. |
| `progressText?` | `string` | Progress words, such as 240 / 500 XP; empty uses a percent. |
| `premiumUnlocked?` | `boolean` | The game owns the premium entitlement. Without it every premium reward reads Locked. Default false. |
| `upgradeLabel?` | `string` | Optional upgrade action while premium is not unlocked (Get premium); no purchase logic. |
| `rewards?` | `ReadonlyArray<{ id: string; tier: string; premium: boolean; name: string; icon: string; amount: string; detail: string; state: string }>` | Keyed rewards in the game's order: { id, tier, premium, name, icon, amount, detail, state }. state locked/available/claimed. Premium rewards fill the premium lane, the rest the free lane; each reward is one tile, and its tier and state are always written in words. Default []. |
| `loading?` | `boolean` | Loading the track; no reward action is available. Default false. |
| `error?` | `string` | A failure explanation with retry; empty is no error. |
| `disabled?` | `boolean` | Prevent claims and upgrades while the game processes an operation. Default false. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |
| `onUpgrade?` | `(e: UiEvent, key?: string) => void` | The upgrade event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `premium`.

Actions: `claim` (Claim the available reward by stable key), `upgrade` (Inspect the premium upgrade), `retry` (Retry loading the track).

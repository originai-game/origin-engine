<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaRoundHud"></a>
### `ArenaRoundHud`

The round's HUD for two teams: each team's name and score on a plate (the local team's marked You), the round and its total, the round's timer, and a banner when the game decides the round's outcome. Read only.

`import { ArenaRoundHud } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/round-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `round?` | `number` | The round being played, as the game counts it. |
| `rounds?` | `number` | How many rounds the match has (Round 3 of 7); 0 hides the total. |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `scoreA?` | `number` | The first team's score (rounds won) as the game counts it. |
| `scoreB?` | `number` | The second team's score as the game counts it. |
| `localTeam?` | `string` | The local player's team, a or b (its plate is marked You); empty is a spectator. |
| `timeLeft?` | `number` | Seconds left in the round as the game counts them (a HudTimer); negative hides it. Default -1. |
| `timeTotal?` | `number` | The round's whole time, for the timer's ring (0: no ring). |
| `outcome?` | `string` | The round's end as the game decided it, for the local side: won, lost, draw, or none (no banner). Default none. |
| `at?` | `number` | The banner's id: a new value replays its entrance. |
| `detail?` | `string` | A line under the banner as the game words it (Flawless, Match point); empty hides it. |

Pack slots: `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `won`, `lost`, `draw`, `critical`, `overflow`.

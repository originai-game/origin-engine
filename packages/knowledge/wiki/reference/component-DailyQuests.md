<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DailyQuests"></a>
### `DailyQuests`

Today's quests: a band with the title and when they refresh (the game counts it down), then one entry per quest: its icon in an octagon socket, its name over a leaning bar with the count beside it, and its prize in a dark box at the end. A done quest is lit gold, its prize box glows and a gold Claim stands beside it; a claimed one dims behind a Claimed stamp. On a narrow list the socket goes and the prize and Claim take a line of their own. The game owns progress, rewards and the clock; the list reports claim (a quest's key).

`import { DailyQuests } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/daily-quests.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Daily quests). |
| `refresh` | `string` | When the quests refresh, as the game words it (Resets in 4h 12m); empty hides it. The game counts it down. |
| `quests` | `ReadonlyArray<{ id: string; name: string; icon: string; progress: number; progressText: string; reward: string; rewardIcon: string; state: string }>` | Today's quests in the game's order: keyed rows of { id, name, icon, progress, progressText, reward, rewardIcon, state }: progress 0 to 1 and progressText (3 / 5); reward the prize in words (50) with its icon; state active, done (its reward waiting) or claimed. |
| `loading?` | `boolean` | Today's quests are on their way: a spinner stands in. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `overflow`.

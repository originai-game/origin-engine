<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Achievements"></a>
### `Achievements`

The achievement list: a band with the title, then the overall count as a large figure over a leaning gold bar; then one entry per achievement, a card with two corners cut: its medal (a hexagon rimmed in its tier's colour), its name, description, its state in words (locked, a bar for one in progress, the date for one unlocked) and its tier on a chip; at the end the share of players who have it, large, and a gold Claim while a reward waits (Claimed once taken). A hidden achievement shows as one, hatched, with neither name nor description, until it is unlocked. On a narrow list the rarity takes a line under the words and Claim takes a line of its own. The game owns every number; the list reports claim (an achievement's key). It fills its height and scrolls inside it.

`import { Achievements } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/achievements.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Achievements). |
| `summary` | `string` | The overall count as the game words it (34 / 120); empty hides it with its bar. |
| `done` | `number` | The overall share unlocked, 0 to 1, for the summary bar. |
| `achievements` | `ReadonlyArray<{ id: string; name: string; about: string; icon: string; state: string; progress: number; progressText: string; date: string; hidden: boolean; rarity: string; tier: string; claimable: boolean; claimed: boolean }>` | The achievements in the game's order: keyed rows of { id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }: state locked, progress or unlocked; progress 0 to 1 and progressText (3 / 10) for one in progress; date when it was unlocked, as the game formats it, empty says Unlocked alone; hidden a secret one, shown as Hidden achievement until unlocked; rarity the share of players who have it (2.4%), empty hides it; tier common, rare, epic or legendary, shown as a word and as the medal's colour, empty hides the word; claimable a reward waiting; claimed it was taken, which hides Claim. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`.

Fixtures: `default`, `empty`, `overflow`.

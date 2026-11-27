<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DailyRewards"></a>
### `DailyRewards`

A login reward calendar: a grid of day tiles with two corners cut, each a header strip with its day, the reward's icon large in the middle, its amount, and a foot with its state in words and a mark. Today is lit gold (a gold header and foot, a glow round the tile); a day taken carries a check stamp over its dimmed art; a missed day is hatched with its word in danger; a milestone is violet with a ribbon. The game owns the schedule, the rewards and the clock; the calendar reports claim (today's key) while canClaim holds: only then is today's tile a button, its Claim foot pulsing with the highlight's aura, sheen and stars; every other day is a labelled tile that takes no focus and reports nothing. A day shows the word of the state the game gave it. Seven across on a desktop, four on a narrow panel, three on a phone, where a last day alone on its row takes the whole row.

`import { DailyRewards } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/daily-rewards.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The calendar's heading; empty takes the group's (Daily rewards). |
| `refresh` | `string` | When the next day's reward opens, as the game words it; empty hides it. |
| `days` | `ReadonlyArray<{ id: string; label: string; icon: string; amount: string; state: string; milestone: boolean }>` | The calendar's days in order: keyed rows of { id, label, icon, amount, state, milestone }: label the day in words (Day 1); amount the reward in words (x100); state claimed, today, upcoming or missed, each shown by its own word: the game sends claimed for today once its reward is taken; milestone a big reward, lit violet with a ribbon. |
| `canClaim?` | `boolean` | Today's reward is waiting: the day whose state is today is a Claim button; otherwise no day is a button. |
| `compact?` | `boolean` | Its place is narrow (under about 300 px): the days (their state words go) tighten. The place decides, not the screen. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`.

Fixtures: `default`, `disabled`, `overflow`.

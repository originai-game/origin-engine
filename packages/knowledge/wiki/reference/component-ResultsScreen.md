<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ResultsScreen"></a>
### `ResultsScreen`

After a match: the verdict set large in a lit fill (gold for a win, red for a loss) over the dimmed scene, a rule with a lit diamond, the level in an octagon beside the XP gained and the leaning progress bar, the grade on a hexagon, then the tabs. Summary shows each stat on a cut plate (a best in gold) and each reward as octagon art with its amount on a tag and its name under it. The page scrolls over the actions.

`import { ResultsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `won?` | `boolean` | The player won. |
| `grade` | `string` | The grade (S, A, B …; empty: none). |
| `title` | `string` | The result's line (Victory, Stage clear). |
| `stats` | `ReadonlyArray<{ best: boolean; count?: boolean; id: string; name: string; value: number; format?: string; decimals?: number }>` | The stats: { name, value, best (bool), format (number or percent; defaults to number), count (bool), decimals }. A stat with count counts up from 0 as the screen opens, written whole or with decimals digits after the point; the others show their value as it is, fractions kept. |
| `level?` | `number` | The level after the match; absent, no level, XP or progress is drawn (a game without levels). |
| `xp?` | `number` | Progress to the next level after the match, 0 to 1. |
| `gained?` | `number` | XP gained. |
| `rewards` | `ReadonlyArray<{ amount: number; id: string; name: string; texture: Texture \| 0; icon?: string }>` | The rewards: { name, amount, texture, icon? }: texture the reward's picture filling its well; 0 shows icon (an atlas icon name, as a Shop price's) on the well's plate. |
| `tab` | `string` | summary, graphs or board. |
| `celebration?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the celebration snippet). |
| `graph?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the graph snippet). |
| `scores?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the scores snippet). |
| `onAgain?` | `((e: UiEvent, key?: string) => void) \| undefined` | The again event, with the row's key where it comes from a row. |
| `onContinue?` | `((e: UiEvent, key?: string) => void) \| undefined` | The continue event, with the row's key where it comes from a row. |
| `onReplay?` | `((e: UiEvent, key?: string) => void) \| undefined` | The replay event, with the row's key where it comes from a row. |
| `onTab?` | `(e: UiEvent, value: string) => void` | The tab switched (onTab), the tab's value (summary, graphs or board). Without it no tab row is drawn, and the page shows `tab`. |

Pack slots: `Count.roll`, `List.cascade`, `List.item`, `Panel.frame`, `Reward.pop`, `Screen.frame`, `Tab.indicator`.

Fixtures: `default`.

Actions: `confirm` (Continue).

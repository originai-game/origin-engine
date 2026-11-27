<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingResults"></a>
### `RacingResults`

The classification after the flag: place, driver, time or gap, points and the fastest-lap mark per row, the player's row marked You, cars that did not finish or were disqualified named so. The game computes every time and point. Continue and Retry (also the details action, X or pad Y) report once; a Retry the game refuses is disabled.

`import { RacingResults } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading (the event's name); empty takes the kit's (Results). |
| `rows?` | `ReadonlyArray<{ id: string; position: number; name: string; time: string; points: string; fastest: boolean; player: boolean; state: string }>` | The finishing order as the game classifies it, keyed rows of { id, position, name, time, points, fastest, player, state }: time the race time or gap as the game words it; points as text (empty hides it); fastest marks the fastest lap; player the local player's row; state finished, dnf or dsq (dnf and dsq are named in words and show no place or time). |
| `bestLap` | `string` | The race's fastest lap time, formatted; empty hides the line under the table. |
| `bestBy` | `string` | Who set the fastest lap; empty leaves the name out of the line. |
| `canRetry` | `boolean` | Retry is offered (false: disabled). |
| `hasPlayer` | `boolean` | A row is the player's: the hero beside the tower shows its place, time and points; false hides it. |
| `onRetry?` | `(e: UiEvent) => void` | Retry (also the details action, X or pad Y). |
| `onContinue?` | `(e: UiEvent) => void` | Continue. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `empty`, `disabled`, `overflow`.

<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PuzzleBoosters"></a>
### `PuzzleBoosters`

The booster tray: each booster a round socket with its art, how many the player holds on a tag at its foot and its name under it. The one the game has armed is lifted, ringed and tagged Armed; one the game will not take is disabled, its art dimmed, tagged in words (Unavailable), and one the player has none of shows a gold + where its count was. Reports choose (a booster's key; tap, click, Enter or pad A); the game decides whether it applies, spends it and arms or fires it. Many boosters scroll across the tray; a narrow tray wraps them onto more rows.

`import { PuzzleBoosters } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/boosters.tsx`).

| Prop | Type | What |
|---|---|---|
| `boosters?` | `ReadonlyArray<{ id: string; icon: string; name: string; count: number; available: boolean }>` | The boosters in tray order, keyed rows of { id, icon, name, count, available }: count how many the player holds, as the game counts them; available false when the game will not take it now (none left, not allowed on this level, a move in progress): the booster is disabled, sends nothing and says so. |
| `armed` | `string` | The booster the game has armed (waiting for its target on the board); empty is none. Shown pressed and worded Armed. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `overflow`.

Actions: `choose` (Use).

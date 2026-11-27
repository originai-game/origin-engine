<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardReward"></a>
### `CardReward`

Pick one of the game's offers: cards drawn as cards, relics as a gold plate with their mark, kind, name and text, and a skip button. The game rolls the offers and applies the pick; the screen reports pick (an offer's key: tap, Enter, pad A) and skip (the button, Escape, pad B). An offer the game refuses is disabled and dimmed; the one the player is looking at wears brackets. The offers share one row while it fits; a phone shows one at nearly full width, so its text stays whole, with the next one peeking at the faded edge, and scrolls sideways.

`import { CardReward } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/reward.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The heading as the game words it (Choose a reward, Treasure). |
| `kicker?` | `string` | A small line over the title naming the moment (Victory · Floor 7); empty hides it. |
| `choices` | `ReadonlyArray<{ id: string; relic: boolean; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; enabled: boolean }>` | The offers the game rolled, keyed rows of { id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }: relic true draws a relic (its icon, typeLabel or Relic, name and text) instead of a card; enabled false when the game will not let it be taken. |
| `selected` | `string` | The offer the player is looking at (focused or tapped once); empty is none. |
| `skipLabel` | `string` | The skip button's text (Skip, Take nothing); empty hides it (Back still reports skip; a game that offers none ignores it). |
| `onPick?` | `(e: UiEvent, key?: string) => void` | The pick event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `Plate.frame`, `Reward.pop`.

Fixtures: `default`, `relics`, `empty`.

Actions: `back` (Skip).

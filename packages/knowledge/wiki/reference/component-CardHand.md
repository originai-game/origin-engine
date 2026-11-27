<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardHand"></a>
### `CardHand`

The player's hand: cards side by side, overlapping as the hand grows so every card keeps a visible edge, turned along a fan when the game asks for one. The selected card rises; when the game says it is playable a Play plate sits on it. A card the game says cannot be played is not disabled (it can still be picked up and read) but carries the unplayable mark and data-playable=false, and never reports play. Reports select (a card's key: tap, Enter, pad A) and play (the Play plate, or X / pad Y on a focused playable card, its key). The game decides legality, cost and targets.

`import { CardHand } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/hand.tsx`).

| Prop | Type | What |
|---|---|---|
| `cards` | `ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; playable: boolean; at: number; reason?: string }>` | The hand in the game's order, keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, playable, reason, at }: playable is the game's answer (cost, targets, rules), never the hand's; reason the game's words for an unplayable card (No energy), empty Cannot be played; at is the card's place across a fanned hand from -1 (first) to 1 (last), 0 the middle, which the game derives from the order. |
| `selected` | `string` | The id of the card the player picked up (it rises and offers Play when playable); empty is none. |
| `fan` | `number` | How far a fanned hand turns its end cards, in degrees (the game's choice, 0 a flat row). |
| `label` | `string` | What a screen reader calls the hand; empty is Hand. |
| `onPlay?` | `(e: UiEvent, key?: string) => void` | The play event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `single`, `disabled`, `overflow`.

Actions: `details` (Play).

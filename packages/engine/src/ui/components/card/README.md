# card: built-in components

The hand and its piles, the collection and deck builder and the reward
choice of a deck-building or card-battle game. They draw what the game
tells them and report intents with stable row keys. They never compute
costs, damage, legality, targets, rewards or deck rules. The hand, the
collection and the reward wrap `CardFace`; a run map is a `NodeGraph`
(`panels/node-graph`).

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `CardCollection` | `card/collection` | The card library of a deck builder. |
| `CardDeckList` | `card/deck-list` | The deck being built. |
| `CardFace` | `card/face` | One card's face. |
| `CardHand` | `card/hand` | The player's hand. |
| `CardPiles` | `card/piles` | The draw, discard and exhaust piles as buttons. |
| `CardReward` | `card/reward` | Pick one of the game's offers. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only card/` draws every member's fixtures under every pack.

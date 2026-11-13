# card: built-in components

The hand and its piles, the collection and deck builder, the reward choice
and the run map of a deck-building or card-battle game, as documents over
core controls (docs/ui.md §4). They draw what the game tells them and
return intents with stable row keys. They never compute costs, damage,
legality, targets, rewards, deck rules or the map's routes.

They are built into the engine: a game's page uses one by name
(`<CardHand .../>`) and loading the page loads it, with the `CardFace` and
`NodeGraph` it uses. `og ui eject card/hand` copies one into
`assets/ui/card/` to change it; the game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing,
and at most a token background on a frame. It sets no colour, font or
art, and it lands in the `og-components` layer. A style pack (`og-pack`,
above it) supplies every appearance through the members' named parts
(docs/ui.md §3.2), qualified by their states and `data-*` attributes, for
example `CardFace.frame[data-rarity=legendary]` or
`button[aria-selected=true]::part(CardHand\.card)`. Each member lists its
parts in its `<meta name="parts">`. The fantasy pack's `card.css`
(`templates/styles/fantasy/`) is the reference dress.

Hooks for a pack: `data-type` (attack, skill, power, status, curse, relic,
or the game's word) and `data-rarity` (common, uncommon, rare, epic,
legendary, mythic) on a card's face and on the buttons that wrap it;
`data-playable` on hand slots and cards; `data-selected` on hand slots and
deck rows; `data-kind` (draw, discard, exhaust) and `data-empty` on piles;
`data-can-add`, `data-locked` and `data-in-deck` on collection cells;
`data-over` and `data-under` on the deck size; `aria-current` on the deck
row shown in detail. A state is never only a colour: an unplayable card
carries a lock badge, a collection card the game will not add a lock when
it is not owned and a cross when it is, with the reason in its
description, an empty pile its 0, an over-full deck its figures.

## Members

| Document | Inputs (props) | Intents |
| --- | --- | --- |
| `CardFace` | `name`, `cost` (text; empty hides the seal), `icon`, `art` (texture; 0 shows the icon), `type`, `typeLabel`, `rarity`, `text` | none |
| `CardHand` | `cards` keyed `{ id, name, cost, icon, art, type, typeLabel, rarity, text, playable, at }` (`at` the card's place across the fan, -1 to 1), `selected`, `fan` (degrees at the ends, 0 a flat row), `label` | `select` (card), `play` (the Play plate on the selected playable card, or X / pad Y on a focused playable card; an unplayable card never sends it) |
| `CardPiles` | `piles` keyed `{ id, kind, label, icon, count, enabled }`, `label` | `open` (pile; a disabled pile sends nothing) |
| `CardCollection` | `title`, `filters` keyed `{ id, label }`, `filter`, `cards` keyed `{ id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }` (`reason` the game's words for a card it will not add), `selected`, `empty` | `add` (card; one the game will not add never sends it), `inspect` (X / pad Y on the focused card, and a tap on one the game will not add), `choosefilter` (filter), `tabNext`/`tabPrev` (E/Q, bumpers) |
| `CardDeckList` | `title`, `entries` keyed `{ id, name, cost, type, rarity, copies, canAdd, canRemove }`, `total`, `limit` (0 hides the size), `selected`, `confirmLabel`, `canConfirm` | `choose` (row), `add` and `remove` (a row's plus and minus; X / pad Y in the focused row is its minus; a refused step is disabled and sends nothing), `confirm` |
| `CardReward` | `title`, `choices` keyed `{ id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }`, `selected`, `skipLabel` | `pick` (offer; a refused one is disabled), `skip` (the button, Escape, pad B) |
| `CardRunMap` | `title`, `floor`, `nodes` and `links` (NodeGraph's: a room's `state` is done, current, available or locked, its kind its `icon` and `shape`), `view`, `zoom`, `panX`, `panY`, `selected`, `legend` keyed `{ id, icon, label }` | NodeGraph's `pick` (room), `zoom`, `pinch`, `pan` |

| Document | Parts |
| --- | --- |
| `CardFace` | `frame`, `cost`, `art`, `name`, `type`, `text` |
| `CardHand` | `frame`, `slot`, `card`, `badge` (Play is a core primary button) |
| `CardPiles` | `frame`, `pile`, `count`, `label` |
| `CardCollection` | `frame`, `band`, `title`, `filter`, `cell`, `badge` |
| `CardDeckList` | `frame`, `band`, `title`, `size`, `row`, `cost`, `name`, `copies`, `step` (Confirm is a core primary button) |
| `CardReward` | `frame`, `band`, `title`, `choice`, `relic` (Skip is a core button) |
| `CardRunMap` | `frame`, `band`, `title`, `floor`, `board`, `legend`, `row` |

Every member carries a `fixture:default` and the state fixtures that apply
to it (`empty`, `single`, `disabled`, `overflow`, `full`, `relics`); the
sheet and the tests mount them. The hand overlaps its cards as it grows,
each slot shrinking while the last keeps the whole card, so eleven cards
fit a phone; the collection's grid goes from five columns to four and three
with its container; the reward's offers share a row no taller than the
room left under the title, and on a phone wrap in thirds, and with no
offers the reward says so. The core `DataGrid` is a table of text rows, so
the collection draws its own grid of faces.

## Review sheet and tests

`og ui sheet card` renders the three review pages (table: hand and piles;
builder: collection and deck list; run: run map and reward) from
`packages/engine/sheets/card/`, dressed by the fantasy pack, with ja and ar
string tables for the locale cells.

`bun test tests/card.test.ts` (from `packages/engine`) mounts every member
by name from a game page from each of its fixtures at desktop and phone,
checks the intents and their keys, that what the game refuses (an
unplayable card, a card it will not add, a removal, an offer, a locked
room) sends nothing, keyed selection through a reorder, keyboard and pad
focus, and that each member's named parts are drawn.

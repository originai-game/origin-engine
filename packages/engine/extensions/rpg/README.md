# rpg: genre kit

Inventory and equipment, journal and quest tracker, dialogue, party and
turn order, the world map and the compass, as documents over core controls
(docs/ui.md §4). It serves party RPGs and open-world games alike: the
tracker and the compass are the open-world HUD, the journal and map their
menus. The kit draws what the game tells it and returns intents with
stable ids. It never computes damage, rolls, loot, prices, weight,
discovery, travel or quest state.

Install it with `og ui add rpg`. Load each document with
`og.load.document(world, "ui-kits/rpg/<file>.svelte")`.

## Information-structure references (§6.1)

Each reference is used for what is on the page and how it is ordered,
never for its art, brand or layout.

| Page | Game, platform | Screenshot source | What we take |
| --- | --- | --- | --- |
| Inventory, equipment, party, turn order, dialogue | Baldur's Gate 3, PC | Game UI Wiki, "Baldur's Gate 3" (gameuiwiki.com/index.php/Baldur's_Gate_3: `BaldursGate3 HUD.png`, `Battle.png`, `DialogueChoice.png`, the inventory and character screens) | Inventory as a slot grid by category with weight carried over capacity and the purse; an item card with rarity-coloured name, type line, numbers and their comparison with what is worn; the character between columns of equipment slots; party portraits with health down one edge; initiative as a strip of portraits, the one acting enlarged, sides marked; dialogue as the speaker's line over numbered replies, a check (skill and DC) beside a reply, taken replies greyed |
| Journal, tracker, map, compass | The Elder Scrolls V: Skyrim, PC | Game UI Database, "The Elder Scrolls V: Skyrim" (gameuidatabase.com/gameData.php?id=287: Codex & Journal, Area Map, Item/Ability Selection Menu and HUD captures) | Quests grouped and listed with the active one marked, its entry text above its objectives, done objectives struck; a one-quest tracker on the HUD; a map whose places are markers with a chosen place's card and fast travel; a compass strip of letters and place markers at the top of the HUD |

## Members

| Document | Inputs (props) | Intents |
| --- | --- | --- |
| `RpgInventory` | `title`; `filters` keyed `{ id, label, count }`, `filter`; `items` keyed `{ id, name, icon, quantity, rarity, kind, equipped, fresh, locked, broken }`, `selected`; `weight`, `load` 0–1 (above 1 overburdened), `gold`; the card: `detailName`, `detailIcon`, `detailRarity`, `detailKind`, `detailText`, `detailStats` keyed `{ id, label, value, delta, change }`, `detailValue`, `canUse`, `canEquip`, `equipLabel`, `canDrop` | `choose` (row key: a tap, Enter or A on an item), `category` (row key), `categoryNext`, `categoryPrev` (E/Q, the bumpers), `use`, `equip` (also X or pad Y), `drop` |
| `RpgEquipment` | `name`, `subtitle`, `portrait` (texture); `slots` keyed `{ id, label, icon, name, rarity, empty, locked }`, `selected`; `stats` keyed `{ id, label, value, delta, change }` | `choose` (row key), `unequip` (also X or pad Y) |
| `RpgJournal` | `title`; `filters` keyed `{ id, label, count }`, `filter`; `quests` keyed `{ id, name, region, level, kind (main, side, task), tracked, fresh, state (active, done, failed) }`, `selected`; the entry: `detailName`, `detailMeta`, `detailText`, `objectives` keyed `{ id, text, progress, done, failed, optional }`, `rewards` keyed `{ id, icon, label }`, `tracked`, `canTrack`, `canLocate` | `choose`, `category` (row keys), `categoryNext`, `categoryPrev`, `track` (also X or pad Y), `locate` |
| `RpgTracker` | `quest`, `distance`; `objectives` keyed `{ id, text, progress, done, optional }`; `updated` (an objective id: a new id replays its mark) | none |
| `RpgDialogue` | `speaker` (empty: narration), `epithet`, `portrait` (texture), `line`; `choices` keyed `{ id, key, text, check, seen, locked, ends }`, `selected`; `canContinue`, `canLog` | `choose` (row key), `advance` (Continue, Enter or A, with no replies), `log` (also X or pad Y) |
| `RpgParty` | `members` keyed `{ id, name, portrait, level, health, healthMax, resource, resourceMax, status, active, down }`, `selected`, `orientation` (vertical: a column beside the screen; horizontal: a row of portrait cards for a phone), `compact` | `choose` (row key) |
| `RpgTurnOrder` | `round`; `entries` keyed `{ id, name, portrait, side (ally, enemy, neutral), current, done }` in initiative order, `selected` | `choose` (row key, to inspect) |
| `RpgWorldMap` | `map` (texture), `region`; `markers` keyed `{ id, x, y (0–1, as the game projects them), icon, label, kind (player, quest, waypoint, town, danger), tracked, discovered }`; `filters` keyed `{ id, label, icon, on }`; `selected`, `detailName`, `detailText`, `canTravel`, `canTrack` | `choose`, `filter` (row keys), `travel` (also X or pad Y), `track`, `center` |
| `RpgCompass` | `facing`; `points` keyed `{ id, x, label }`; `markers` keyed `{ id, x, icon, kind, distance, above, below }`, x 0–1 across the strip as the game projects it | none |

Every list is a keyed collection, and selection is by key, so it holds
through a reorder, a removal and a restore. A disabled control sends
nothing, from a tap or from its shortcut. Each member says so when it is
empty: an empty bag, journal or party has a status line. The tracker and
turn order are hidden when there is nothing to show. Under a coarse
pointer the keycaps drop out. Lists scroll inside their panel, so the
primary action stays on screen at 150 % text and on a 390 px phone. Names
wrap and never truncate.

## Parts (dressed by packs)

Frames: `Inventory.frame`, `Inventory.card` (the item card),
`Equipment.frame`, `Journal.frame`, `Journal.entry`, `Dialogue.frame`,
`Map.frame`, `Map.place` (the chosen place's card), `Tracker.frame` and
`TurnOrder.frame`; without a dressing they take the core tokens (`--panel`,
`--panel-raised`, `--well`). Rows and cells:
`Inventory.item[uncommon|rare|epic|legendary|mythic][broken]` (a button
for the grid, a box for the card's art), `Inventory.category`,
`Equipment.slot[empty]`, `Equipment.portrait`, `Journal.quest[main][done][failed]`,
`Journal.category`, `Journal.objective[done][failed][optional]`,
`Tracker.frame`, `Tracker.objective[done][optional][updated]`,
`Dialogue.frame`, `Dialogue.speaker`, `Dialogue.choice[seen][ends][checked]`,
`Party.member[active][down]`, `Party.portrait`, `Party.health`,
`Party.resource` (meters: `Meter.fill[region=…]` colours them),
`TurnOrder.frame`, `TurnOrder.entry[ally|enemy|neutral][current][done]`,
`Map.frame`, `Map.marker[player|quest|waypoint|town|danger][unknown][tracked]`,
`Map.legend`, `Compass.frame` and `Compass.marker[quest|waypoint|danger|ally]`.
Buttons take hover, pressed, focused, selected and disabled from the
control. The kit draws rarity as a band at the foot of a slot and
selection as a gold ring inside it, over whatever frame a pack paints, so
neither rests on colour alone or on a pack. Rarity and side colours come
from tokens (`--uncommon` …
`--mythic`, `--team1`, `--danger`, `--gold`). Frames use `-og-frame:
panel`, `card`, `slot` and `tooltip`, which a painted pack fills.

## Pages and frames (§6.2)

`og ui sheet rpg` renders `sheet/`, the kit installed as a game installs
it, dressed by `fantasy`.

- Normal: `inventory` is the bag beside the equipment: categories, a
  grid with every rarity, equipped, new, locked and broken items, the load
  and purse, and the selected item's card with its comparison.
- Stress: `journal` is the party (a downed, a poisoned and a
  concentrating member) with the tracker and compass under it, the turn
  order over the world map (every marker kind, an undiscovered place, a
  chosen waypoint), a conversation with a check, a locked, a taken and a
  leaving reply, and the journal (a long quest name, a failed and a done
  quest, optional and failed objectives, rewards). On a phone they stack
  and scroll, the party as a row of cards.
- The primary pack is `fantasy`. The contrast pack, `pixel`, is pending
  because pixel is unbuilt.

## Tests

`tests/rpg.test.ts` mounts every member from named stores, dressed by
fantasy. It covers intents by row key from touch, keyboard and pad,
disabled rows and refused actions sending nothing (shortcuts included),
the empty outcomes, selection through a reorder, a removal and a restore,
the initiative order, markers centred where the game projects them, and
every page at 390 px with nothing past its width and 44 px touch targets.

# rpg: built-in components

Inventory, character and equipment, quests, the world map, conversation and
the turn-based and exploration HUDs, as documents over core controls
(docs/ui.md §4). They draw what the game tells it and returns intents
with stable row keys. It never computes stats, damage, turn order, legality
or loot.

They are built into the engine: a game's page uses one by name
(`<RpgBackpack .../>`) and loading the page loads it, with the `Minimap`
it uses. `og ui eject rpg/backpack` copies one into `assets/ui/rpg/` to
change it; the game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing. It
sets no colour, font or art, and it lands in the `og-components` layer. A style pack
(`og-pack`, above it) supplies every appearance with ordinary selectors on
the members' root classes, their classes and their `data-*` attributes, for
example `.rpg-backpack .cell[data-rarity=epic]`. `::part()` is used only for
the core parts the registry lists (`button.command::part(base)`). The
fantasy pack (`templates/styles/fantasy/`) is the reference dress.

Hooks for a pack: `data-rarity` (common, uncommon, rare, epic, legendary,
mythic) on backpack and equipment cells and on the item card;
`data-equipped`, `data-fresh`, `data-locked` and `data-broken` on cells;
`data-tone` on combat log lines (damage, heal, status, crit, plain) and on
enemy effects (harm, boon, weak, strong); `data-plate` (large, compact) and
`data-down` on vitals; `data-side`, `data-acting` and `data-down` on turns;
`data-kind` on map markers, legend entries and compass marks; `data-ready`
on skills; `data-done`, `data-tracked` and `data-fresh` on quests.

## Members

| Document | Inputs (props) | Intents |
| --- | --- | --- |
| `RpgBackpack` | `title`, `tabs` keyed `{ id, label }`, `tab`, `items` keyed `{ id, name, icon, count, rarity, equipped, fresh, locked, broken }`, `selected`, `weight`, `load` 0–1, `gold` | `choose` (cell), `choosetab` (tab), `tabNext`/`tabPrev` (E/Q, bumpers) |
| `RpgItemCard` | `name`, `icon`, `art` (texture), `rarity`, `kind`, `lead` and `stats` keyed `{ id, icon, label, value, delta, trend, current }`, `lore`, `primary`, `secondary`, `canPrimary`, `canSecondary` | `equip`, `discard` |
| `RpgCharacter` | `title` (a heading band; empty shows the name header), `name`, `line`, `portrait`, `figure`, `slots` keyed `{ id, side (start, end), label, icon, rarity }`, `selected`, `attributes` keyed `{ id, group (a, b), icon, label, value }` | `choose` (slot), `unequip` (details: X, pad Y) |
| `RpgStats` | `title`, `groups` keyed `{ id, column (a, b) }`, `rows` keyed `{ id, group, icon, label, value }` | none |
| `RpgMenuBar` | `tabs` keyed `{ id, label }`, `tab` | `choosetab`, `tabNext`/`tabPrev`, `close` (button, Escape, B) |
| `RpgQuestLog` | `title`, `groups` keyed `{ id, label, done }`, `quests` keyed `{ id, group, name, tracked, fresh }`, `selected` | `choose`, `track` (details) |
| `RpgQuestDetail` | `name`, `summary`, `objectives` keyed `{ id, text, done, optional, step }`, `rewards` keyed `{ id, icon, label, detail }`, `scenery` | none |
| `RpgQuestTracker` | `name`, `steps` keyed `{ id, text, done }`, `distance` (empty hides it) | none |
| `RpgWorldMap` | `map` (texture), `markers` keyed `{ id, kind, x, y, label }`, `selected`, `legend` keyed `{ id, kind, label }`, `here` | `choose` (marker; the player's own is not a target) |
| `RpgPlaceCard` | `name`, `about`, `picture`, `marked`, `canTravel` | `mark`, `travel` |
| `RpgConversation` | `speaker`, `portrait`, `line`, `replies` keyed `{ id, number, text, used }`, `selected` | `reply` (row key), `next` (Continue when there are no replies) |
| `RpgCompass` | `heading`, `marks` keyed `{ id, kind, label, at }` | none |
| `RpgVitals` | `name`, `portrait`, `level`, `health`, `healthMax`, `mana`, `manaMax`, `plate` (large, compact), `down` | none |
| `RpgParty` | `members` keyed `{ id, name, portrait, level, health, healthMax, mana, manaMax, down }`, `selected` | `choose` |
| `RpgTurnOrder` | `round`, `turns` keyed `{ id, name, portrait, side, acting, down }` | none |
| `RpgActionBar` | `skills` keyed `{ id, icon, key, name, cooldown 0–1, ready }`, `items` keyed `{ id, icon, key, name, count }` | `use` (row key; a skill that is not ready is disabled) |
| `RpgCombatLog` | `lines` keyed `{ id, text, tone, detail }`, oldest first | none |
| `RpgEnemyInfo` | `name`, `portrait`, `level`, `health`, `healthMax`, `effects` keyed `{ id, icon, label, tone }` | none |
| `RpgCommands` | `commands` keyed `{ id, label, icon, enabled }`, `selected` | `choose` (row key; a disabled command sends nothing) |

Every member carries a `fixture:default` the sheet and the tests mount it
from. Phone layouts come from the members' own media and container queries:
the backpack's and menu bar's tabs scroll behind an edge fade, the item card
shrinks its hero so two stat rows show, the commands take two rows of three
and the enemy info drops its portrait.

## Review sheet and tests

`og ui sheet rpg` renders the five review pages (inventory, journal,
equipment, explore, combat) from `packages/engine/sheets/rpg/`, dressed by
the fantasy pack, with
ja, zh-CN and ar string tables for the locale cells. The sheet's painted
art (portraits, figure, item and skill icons, scene, map) was generated for
it with gpt-image-2.6 through the OriginGame gateway and is sheet fixture
data, not part of the components.

`bun test tests/rpg.test.ts` (from `packages/engine`) mounts every member
by name from a game page at desktop and phone, checks the intents, disabled
states, keyed selection, keyboard and pad focus, and that a pack rule beats
a component rule of equal specificity.

The sheet's fixture text goes through `assets/strings/` (en, ja,
zh-CN, ar) as a game's data would, so pseudo brackets every label. Pack type
is in `em` of the body step so the player's text scale applies.

### References

`references/vulkan/` holds the 15 art-approved cells, recorded on the
GPU host (`references/metal/` keeps the Metal ones): the five
pages at desktopHd, phonePortrait and phoneLandscape, 100 %, game locale.
Capture the same cells and compare, from the repository root:

```sh
P=.amp/in/rpg-sheet; K=packages/engine/sheets/rpg
rsync -a --delete --exclude references $K/ $P/
source scripts/lock.sh; og_gpu_acquire rpg-sheet
S=""; for p in inventory journal equipment combat explore; do S="$S --screen $p=style:fantasy;page:$p"; done
OG_ENGINE_PACKAGE="$PWD/packages/engine" OG_STYLES_DIR="$PWD/templates/styles" \
  og ui preview $P $S --breakpoints desktopHd,phonePortrait,phoneLandscape \
  --scales 100 --locales game --settle 2 --out "$PWD/.amp/in/rpg-capture"
bun $K/verify.ts .amp/in/rpg-capture
```

`verify.ts` rejects a missing cell or any changed pixel. Recording is
manual: review each changed cell before replacing it.

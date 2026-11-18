# rpg: built-in components

Inventory, character and equipment, quests, the world map, conversation and
the turn-based and exploration HUDs, as documents over core controls
(docs/ui.md §4). They draw what the game tells them and return intents
with stable row keys. They never compute stats, damage, turn order, legality
or loot.

They are built into the engine: a game's page uses one by name
(`<RpgBackpack .../>`) and loading the page loads it, with the `Minimap`
it uses. `og ui eject rpg/party` copies one into `assets/ui/rpg/` to
change it; the game's copy of the same name wins.
The backpack, item card, quest log and conversation are in `../panels/`, and
the combat log and quest tracker in `../hud/`: every genre uses them. They
keep their names (`RpgBackpack`…) and their contract here.

The shared panels also include `RpgCrafting`, `RpgTrade`, `RpgLoot` and
`RpgConversationBacklog` (see the RPG sheet README for their game-owned
data and intent contracts). `RpgConversation` optionally exposes history,
skip and auto buttons with `showControls` (default false). `canBacklog`
and `canAuto` default true; `canSkip` and `auto` default false. The
`backlog`, `skip` and `toggleAuto` intents request game decisions only:
the component neither opens history nor skips or advances dialogue itself.
The game feeds its current auto setting back through `auto`.

## Structure and appearance

Each member is HTML with component CSS in the `og-components` layer.
`:host` owns the root dimensions; shared `data-layout` and `data-gap`
primitives arrange the interior. Component CSS keeps sizes and placement.
Optional pieces use `{#if}`. Parts retain their semantic names and end in
shared roles: health and mana `track`/`fill`, count `badge`, `headline`,
`caption`, selection `cell` and quest progress `dot`. Role tokens supply
the appearance and shared motion supplies transitions. A style
pack (`og-pack`, above it) overrides the members' named parts (docs/ui.md
§3.2), qualified by states, for example
`RpgBackpack::cell[data-rarity=epic]`. Each member lists its
parts in its `<meta name="parts">`: its `frame`, and as it has them its
`band` and `title`, `tab`, `cell`, `row`, `name` and the like
(`RpgBackpack.cell`, `RpgQuestLog.row`; every member's are under Parts).
Classes are left for inner structure and state. A pack dresses them
through its tokens (`templates/styles/<pack>/base.css`).

Hooks for a pack: `data-rarity` (common, uncommon, rare, epic, legendary,
mythic) on backpack and equipment cells and on the item card;
`data-equipped`, `data-fresh`, `data-locked` and `data-broken` on cells;
`data-tone` on combat log lines (damage, heal, status, crit, plain) and on
enemy effects (harm, boon, weak, strong); `data-plate` (large, compact) and
`data-down` on vitals; `data-side`, `data-acting` and `data-down` on turns;
`data-kind` on map markers, legend entries; `data-ready`
on skills; `data-done`, `data-tracked` and `data-fresh` on quests.

## Members

| Document | Inputs (props) | Intents |
| --- | --- | --- |
| `RpgBackpack` | `title`, `kicker`, `capacity`, `tabs` keyed `{ id, label, count }`, `tab`, `items` keyed `{ id, name, icon, count, rarity, equipped, fresh, locked, broken }`, `selected`, `weight`, `load` 0–1 (above 1 is overburdened), `gold` | `choose` (cell), `move` with `{ dragged, target }` (a cell dropped on another; a locked one does not drag), `choosetab` (tab), `tabNext`/`tabPrev` (E/Q, bumpers) |
| `RpgItemCard` | `name`, `icon`, `art` (texture), `rarity`, `kind`, `line`, `lead` and `stats` keyed `{ id, icon, label, value, delta, trend, current }`, `lore`, `primary`, `secondary`, `canPrimary`, `canSecondary`, `reason` | `equip`, `discard` |
| `RpgCharacter` | `title` (a heading band; empty shows the name header), `name`, `line`, `portrait`, `figure`, `slots` keyed `{ id, side (start, end), label, icon, rarity }`, `selected`, `attributes` keyed `{ id, group (a, b), icon, label, value }` | `choose` (slot), `unequip` (details: X, pad Y) |
| `RpgStats` | `title`, `groups` keyed `{ id, column (a, b) }`, `rows` keyed `{ id, group, icon, label, value }` | none |
| `RpgMenuBar` | `tabs` keyed `{ id, label }`, `tab` | `choosetab`, `tabNext`/`tabPrev`, `close` (button, Escape, B) |
| `RpgQuestLog` | `title`, `groups` keyed `{ id, label, done }`, `quests` keyed `{ id, group, name, tracked, fresh }`, `selected` | `choose`, `track` (details) |
| `RpgQuestDetail` | `name`, `summary`, `objectives` keyed `{ id, text, done, optional, step }`, `rewards` keyed `{ id, icon, label, detail }`, `scenery` | none |
| `RpgQuestTracker` | `label` (empty: Main quest), `name`, `steps` keyed `{ id, text, done, tag }` (the first not done is the step in play), `distance` (empty hides it) | none |
| `RpgWorldMap` | `map` (texture), `markers` keyed `{ id, kind, x, y, label }`, `selected`, `zoom` (1 fits the box), `panX`, `panY` (CSS px), `legend` keyed `{ id, kind, label }`, `here` | `choose` (marker; the player's own is not a target), `zoom` (wheel, `deltaY`), `pinch` (`scale`), `grab`, `pan`, `release` (a press, its moves and its end, `x` and `y` in the map's box); the game keeps the zoom and the pan |
| `RpgPlaceCard` | `name`, `about`, `picture`, `marked`, `canTravel` | `mark`, `travel` |
| `RpgConversation` | `speaker`, `portrait`, `line`, `rate` (characters a second; 0 shows the line at once), `replies` keyed `{ id, number, text, used }`, `selected`, `showControls`, `canBacklog`, `canSkip`, `canAuto`, `auto` | `reply` (row key), `next` (Continue when there are no replies), `revealed` (a line shown at a rate is whole), `backlog`, `skip`, `toggleAuto` |
| `RpgVitals` | `name`, `portrait`, `level`, `health`, `healthMax`, `mana`, `manaMax`, `plate` (large, compact), `down` | none |
| `RpgParty` | `members` keyed `{ id, name, portrait, level, health, healthMax, mana, manaMax, down }`, `selected` | `choose` |
| `RpgTurnOrder` | `round`, `turns` keyed `{ id, name, portrait, side, acting, down }` | none |
| `RpgCombatLog` | `lines` keyed `{ id, text, tone, detail }`, oldest first | none |
| `RpgEnemyInfo` | `name`, `portrait`, `level`, `health`, `healthMax`, `effects` keyed `{ id, icon, label, tone }` | none |
| `RpgCommands` | `commands` keyed `{ id, label, icon, enabled }`, `selected` | `choose` (row key; a disabled command sends nothing) |
| `RpgLeader` | `name`, `level`, `portrait`, `health`, `healthMax`, `ghost`, `mana`, `manaMax`, `stamina`, `staminaMax`, `segment` (stamina per cell), `low` | none |
| `RpgEffects` | `effects` keyed `{ id, name, icon, glyph, remaining, label, harmful }` | none |
| `RpgCompanions` | `members` keyed `{ id, name, portrait, health, healthMax, ghost, mana, manaMax, key, fallen }`, `low` | none |
| `RpgMinimap` | `map` (texture), `centerX`, `centerY`, `zoom`, `heading`, `blips` keyed `{ id, kind (ally, foe, boss, objective), x, y, label }`, `place`, `clock` | none |
| `RpgLockOn` | `target` (empty: nothing locked) | none |
| `RpgHits` | `hits` keyed `{ id, amount, kind (normal, critical, burn, heal, miss), call, word }`, newest first | none |
| `RpgCombo` | `count`, `word`, `remaining` (the chain's window, 0–1), `multiplier` | none |
| `RpgLootFeed` | `items` keyed `{ id, name, icon, glyph, rarity, label, quantity, currency, fading }`, newest first | none |
| `RpgPrompt` | `key`, `text`, `disabled`, `reason` | none |
| `RpgSkills` | `slots` keyed `{ id, name, icon, glyph, key, kind (skill, item, ultimate), cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }` | none |
| `RpgHints` | `hints` keyed `{ id, key, label }` | none |
| `RpgExperience` | `level`, `value` (0–1 to the next level) | none |

## Parts

Each part's state qualifiers are listed after it: a class
(`.cooling`), an attribute (`[data-rarity]`, `[aria-selected]`) or
`:disabled`. A part that is a button also has the control's own hover,
pressed, focus-visible and disabled states (docs/ui.md §3.2).

| Component (document) | Parts |
| --- | --- |
| `RpgBackpack` (`../panels/backpack.svelte`) | `frame`, `band`, `kicker`, `title`, `purse`, `tab` (a button: `[aria-selected]`), `count`, `capacity`, `cell` (a button: `[aria-selected]` `[data-broken]` `[data-equipped]` `[data-fresh]` `[data-locked]` `[data-rarity]`), `state`, `bracket`, `track`, `fill` |
| `RpgCharacter` (`character.svelte`) | `frame`, `band`, `title`, `name`, `cell` (a button: `[aria-selected]` `[data-empty]` `[data-rarity]`), `row` |
| `RpgCombatLog` (`../hud/combat-log.svelte`) | `frame`, `band`, `title`, `row` (`[data-tone]`), `fade` |
| `RpgCommands` (`commands.svelte`) | `frame`, `band`, `title`, `cell` (a button: `.primary` `[aria-selected]` `:disabled`) |
| `RpgConversation` (`../panels/conversation.svelte`) | `frame`, `name`, `row` (a button: `[aria-selected]` `[data-used]`), `controls`, `backlog`, `skip`, `auto` (`[aria-pressed]`) |
| `RpgEnemyInfo` (`enemy-info.svelte`) | `frame`, `band`, `title`, `name`, `row` (`[data-tone]`) |
| `RpgItemCard` (`../panels/item-card.svelte`) | `frame` (`[data-rarity]`), `hero`, `band`, `kicker`, `name`, `line`, `lead`, `delta` (`[data-trend]`), `current`, `stats`, `row`, `lore`, `actions`, `reason` |
| `RpgMenuBar` (`menu-bar.svelte`) | `frame`, `tab` (a button: `[aria-selected]`) |
| `RpgParty` (`party.svelte`) | `frame`, `cell` (a button: `[aria-selected]` `[data-down]`) |
| `RpgPlaceCard` (`place-card.svelte`) | `frame`, `name` |
| `RpgQuestDetail` (`quest-detail.svelte`) | `frame`, `band`, `title`, `row` (`[data-done]` `[data-optional]` `[data-step]`), `reward`, `scenery` |
| `RpgQuestLog` (`../panels/quest-log.svelte`) | `frame`, `band`, `title`, `heading`, `row` (a button: `[aria-selected]` `[data-done]` `[data-fresh]` `[data-tracked]`) |
| `RpgQuestTracker` (`../hud/quest-tracker.svelte`) | `frame`, `legend`, `quest`, `distance`, `step` (`[data-done]` `[data-current]`), `marker`, `text`, `progress` |
| `RpgStats` (`stats.svelte`) | `frame`, `band`, `title`, `row` |
| `RpgTurnOrder` (`turn-order.svelte`) | `frame`, `cell` (`[aria-current]` `[data-acting]` `[data-down]` `[data-side]`), `title` |
| `RpgVitals` (`vitals.svelte`) | `frame` (`[data-down]` `[data-plate]`) |
| `RpgLeader` (`leader.svelte`) | `frame` (`.low`), `portrait`, `face`, `hero`, `level`, `value`, `max`, `health`, `ghost`, `mana`, `stamina` |
| `RpgEffects` (`effects.svelte`) | `frame`, `effect` (`[data-harmful]`), `tile`, `art`, `timer`, `time` |
| `RpgCompanions` (`companions.svelte`) | `frame`, `member` (`[data-fallen]` `.low`), `portrait`, `face`, `callsign`, `fallen`, `health`, `ghost`, `mana`, `key` |
| `RpgMinimap` (`minimap.svelte`) | `frame`, `scope`, `map`, `blip` (`[data-kind]`), `self`, `north`, `place`, `clock` |
| `RpgLockOn` (`lock-on.svelte`) | `frame` (`[data-locked]`), `ring` |
| `RpgHits` (`hits.svelte`) | `frame`, `hit` (`[data-kind]`), `call`, `amount`, `kind` |
| `RpgCombo` (`combo.svelte`) | `frame`, `count`, `word`, `decay`, `lit`, `bonus` |
| `RpgLootFeed` (`loot-feed.svelte`) | `frame`, `entry` (`[data-rarity]` `[data-fading]`), `art`, `item`, `rarity`, `count` |
| `RpgPrompt` (`prompt.svelte`) | `frame` (`.disabled`), `key`, `action`, `reason` |
| `RpgSkills` (`skills.svelte`) | `frame`, `skill` (`[data-kind]` `[data-ready]` `[data-unaffordable]`), `slot`, `art`, `cooldown`, `edge`, `time`, `reason`, `count`, `charges`, `pip`, `ring`, `key` |
| `RpgHints` (`hints.svelte`) | `frame`, `hint`, `key`, `action` |
| `RpgExperience` (`experience.svelte`) | `frame`, `reading`, `line`, `lit` |
| `RpgWorldMap` (`world-map.svelte`) | `frame`, `marker` (a button: `[aria-selected]` `[data-kind]` `:disabled`), `row` (`[data-kind]`) |

Every member carries a `fixture:default` the sheet and the tests mount it
from. Phone layouts come from the members' own media and container queries:
the backpack's and menu bar's tabs scroll, the item card
shrinks its hero so two stat rows show, the commands take two rows of three
and the enemy info drops its portrait. The action HUD's members (leader to
experience above) stand bare over the scene on the `--hud-*` roles; on a
phone the companions stand side by side, the quest tracker shows the step
in play, the skills drop their consumables and the loot feed its older
rows, and under a touch screen the key caps and hints are hidden.

## Review sheet and tests

`og ui sheet rpg` renders the five review pages (inventory, journal,
equipment, explore, combat) from `packages/engine/sheets/rpg/`, in the
default style (the fantasy pack is also available), with
ja, zh-CN and ar string tables for the locale cells. The sheet's painted
art (portraits, figure, item and skill icons, scene, map) was generated for
it with gpt-image-2.6 through the OriginGame gateway and is sheet fixture
data, not part of the components. The explore page lays out the action HUD as its approved
mockup does, over the HUD art direction's sample stage, map, portraits and
icons (`assets/art/hud-*.jpg`, sheet only).
`og ui review rpg --style scifi` (or `--style cartoon`) renders the same
pages in that pack, dressed by its tokens
(`templates/styles/<pack>/base.css`).

`bun test tests/rpg.test.ts` (from `packages/engine`) mounts every member
by name from a game page at desktop and phone, checks the intents, disabled
states, keyed selection, keyboard and pad focus, and that a pack rule beats
a component rule of equal specificity.

The sheet's fixture text goes through `assets/strings/` (en, ja,
zh-CN, ar) as a game's data would, so pseudo brackets every label. Pack type
is in `em` of the body step so the player's text scale applies.

### References

`references/vulkan/` holds the 15 art-approved cells, recorded on the
GPU host: the five
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

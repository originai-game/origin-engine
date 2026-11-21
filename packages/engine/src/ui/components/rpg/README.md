# rpg: built-in components

Character and equipment, quests, the world map and the turn-based and
exploration HUDs. They draw what the game tells them and report intents
with stable row keys. They never compute stats, damage, turn order,
legality or loot.

Every genre uses the backpack, item card, quest log, conversation and its
backlog, crafting, trade and loot, so they are in `../panels/`, and the
combat log and quest tracker in `../hud/`. `RpgConversation` shows history,
skip and auto buttons with `showControls`; its `backlog`, `skip` and
`toggleAuto` intents ask the game, which opens history, skips or advances
and feeds its auto setting back through `auto`.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `RpgCharacter` | `rpg/character` | A character sheet. |
| `RpgCombo` | `rpg/combo` | The combo counter beside the fight. |
| `RpgCommands` | `rpg/commands` | A turn's commands as a row of plates, each an icon over its label; the selected one is the primary plate. |
| `RpgCompanions` | `rpg/companions` | The leader's companions under the leader's vitals. |
| `RpgEffects` | `rpg/effects` | The buffs and debuffs on the leader, under the vitals. |
| `RpgEnemyInfo` | `rpg/enemy-info` | The targeted enemy. |
| `RpgExperience` | `rpg/experience` | The experience line across the screen's foot. |
| `RpgHints` | `rpg/hints` | Input hints at the screen's foot. |
| `RpgHits` | `rpg/hits` | Damage and healing figures beside the fight. |
| `RpgLeader` | `rpg/leader` | The party leader's vitals at the screen's top corner, built from the RPG HUD's blocks. |
| `RpgLockOn` | `rpg/lock-on` | The lock-on reticle the page centres on the locked target. |
| `RpgLootFeed` | `rpg/loot-feed` | The loot feed low on the screen's side. |
| `RpgMenuBar` | `rpg/menu-bar` | The menu's top bar. |
| `RpgMinimap` | `rpg/minimap` | The minimap at the screen's top corner. |
| `RpgParty` | `rpg/party` | The party as a row of portrait cards. |
| `RpgPlaceCard` | `rpg/place-card` | A place picked on the map, on dark glass. |
| `RpgPrompt` | `rpg/prompt` | What the player can do now, over the ability bar. |
| `RpgQuestDetail` | `rpg/quest-detail` | One quest. |
| `RpgSkills` | `rpg/skills` | The ability bar at the foot of the screen, a row of HudSlot blocks. |
| `RpgStats` | `rpg/stats` | A character's figures in two columns of grouped rows, each an icon, a label and a value. |
| `RpgTurnOrder` | `rpg/turn-order` | The initiative. |
| `RpgVitals` | `rpg/vitals` | A character's vitals. |
| `RpgWorldMap` | `rpg/world-map` | A map with its markers, a one-line legend and an optional caption. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only rpg/` draws every member's fixtures under every pack.

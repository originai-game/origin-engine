# shooter: built-in components

The loadout and the combat HUD (the squad, objectives, radar, round and
score, pickups, damage dealt, the interaction prompt, the effects, the
vitals and the ability bar). They draw what the game tells them and report
intents with stable ids. They never compute damage, spread, reloads, kills
or legality. Team and match scores belong to `arena`. `Crosshair`,
`AmmoCounter`, `DamageDirection` and `Killfeed` are in `../hud/`, since
any game with aiming uses them. The HUD is the approved "smoked glass, lit
edges" look: stroked type over the scene, cut plates and tags, octagon
portraits and slots, leaning bars.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `ShooterAbilities` | `shooter/abilities` | The ability bar at the foot of the screen. |
| `CombatHud` | `shooter/combat-hud` | The shooter's whole HUD. |
| `ShooterDamage` | `shooter/damage` | Damage the player deals, beside the reticle. |
| `ShooterEffects` | `shooter/effects` | The effects on the player, over the vitals. |
| `Loadout` | `shooter/loadout` | A loadout screen over the scene. |
| `ShooterObjectives` | `shooter/objectives` | The objectives at the screen's top corner. |
| `ShooterPickups` | `shooter/pickups` | The pickup feed lower on the screen's side. |
| `ShooterPrompt` | `shooter/prompt` | What the player can do with what is in front of them, under the aim. |
| `ShooterRadar` | `shooter/radar` | The radar at the screen's top corner. |
| `ShooterRound` | `shooter/round` | The round on a plate lit along its top, its time left beside it, and the score with the player's side in its blue against the other's red. |
| `ShooterSquad` | `shooter/squad` | The squad at the screen's top corner. |
| `ShooterVitals` | `shooter/vitals` | The player's vitals at the screen's bottom corner. |

## Information-structure references (§6.1)

Each reference is used for what is on the page and how it is ordered,
never for its art, brand or layout.

| Page | Game, platform | Screenshot source | What we take |
| --- | --- | --- | --- |
| Combat HUD | Apex Legends, PC | interfaceingame.com/games/apex-legends ("Shield", "Eliminated", in-game captures); EA accessibility features, PC (ea.com/able/resources/apex-legends/pc/features) | Ammunition as magazine over reserve beside the weapon name and fire mode; hit confirmation on the reticle (a stronger mark for a headshot, a kill or a broken shield); incoming damage as a direction around the centre; a short killfeed |
| Loadout and killfeed | VALORANT, PC | ONE Esports "Valorant HUD" breakdown (Nigel Zalamea, 2023-07-22): killfeed top right, weapon inventory bottom right; Valorant Wiki "Credits" (armory capture) | Loadout as slots by category (primary, secondary, melee, utility), each with name, cost or ammunition and an equipped state; killfeed rows as killer, weapon, victim, with headshots marked and the local player's rows emphasised |

## Gallery

`bun packages/engine/src/node/gallery.ts --only shooter/` draws every member's fixtures under every pack.

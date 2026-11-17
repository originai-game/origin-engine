# shooter: built-in components

Aim, ammunition, damage direction, killfeed, loadout and the combat HUD
(the squad, objectives, radar, round and score, pickups, damage dealt, the
interaction prompt, the effects, the vitals and the ability bar), as
documents over core controls (docs/ui.md §4). They draw what the game tells it and
returns intents with stable ids. It never computes damage, spread,
reloads, kills or legality. Team and match scores belong to `arena`.
`Crosshair`, `AmmoCounter`, `DamageDirection` and `Killfeed` are filed in
`../hud/`, since any game with aiming uses them; this README keeps their
contract, and the kit holds `CombatHud`, `Loadout` and the `Shooter*` HUD
members. The HUD is drawn as the approved "smoked glass, lit edges" mockup:
stroked type over the scene, cut plates and tag plates, octagon portraits
and slots, leaning bars, all from the default style's `--hud-*` roles.

They are built into the engine: a game's page uses one by name
(`<Killfeed .../>`) and loading the page loads it, with the `StatBar`
that `CombatHud` composes. `og ui eject hud/killfeed` copies one into
`assets/ui/hud/` to change it; the game's copy of the same name wins.

## Information-structure references (§6.1)

Each reference is used for what is on the page and how it is ordered,
never for its art, brand or layout.

| Page | Game, platform | Screenshot source | What we take |
| --- | --- | --- | --- |
| Combat HUD | Apex Legends, PC | interfaceingame.com/games/apex-legends ("Shield", "Eliminated", in-game captures); EA accessibility features, PC (ea.com/able/resources/apex-legends/pc/features) | Ammunition as magazine over reserve beside the weapon name and fire mode; hit confirmation on the reticle (a stronger mark for a headshot, a kill or a broken shield); incoming damage as a direction around the centre; a short killfeed |
| Loadout and killfeed | VALORANT, PC | ONE Esports "Valorant HUD" breakdown (Nigel Zalamea, 2023-07-22): killfeed top right, weapon inventory bottom right; Valorant Wiki "Credits" (armory capture) | Loadout as slots by category (primary, secondary, melee, utility), each with name, cost or ammunition and an equipped state; killfeed rows as killer, weapon, victim, with headshots marked and the local player's rows emphasised |

## Members

| Document | Inputs (props) | Intents |
| --- | --- | --- |
| `Crosshair` | `spread` 0–1; `hit` none, hit, headshot, kill or shieldBreak (the game clears it to none); `hitAt` (a new id replays the mark); `aimed`; `shape` cross, dot or circle | none |
| `AmmoCounter` | `weapon`, `mode` (auto, burst, single), `magazine`, `capacity` (a tick per round; a strip past 60), `reserve`, `reloading` 0–1 (negative: not reloading), `low` (a fraction of capacity), `secondary`, `secondaryAmmo` (negative hides it), `secondaryKey` | none |
| `DamageDirection` | `hits` keyed `{ id, angle (rad, 0 ahead, clockwise), strength 0–1, kind (normal, shield, critical) }` | none |
| `Killfeed` | `rows` keyed `{ id, killer, victim, weapon, kind (rifle, smg, pistol, shotgun, sniper, launcher, melee: its glyph; empty: the weapon's name), headshot, self, team (ally or enemy, the killer's side) }`, newest first; an upright phone shows the newest, one on its side the three newest | none |
| `Loadout` | `title` (empty: the component's Loadout), `categories` keyed `{ id, label }` (an empty label: the component's string for primary, secondary, melee or utility), `slots` keyed `{ id, category, name, icon, detail, equipped, locked, cost }`, `selected` (a slot id) | `select` (row key: a tap, Enter or A on a slot), `equip` (row key: the Equip button sends the selected slot's, X or pad Y the focused slot's), `back` (the Back button, Escape or B) |
| `ShooterVitals` | `name`, `level`, `portrait` (texture; 0: the person glyph), `health`, `healthMax`, `ghost` (health before the latest loss), `shield`, `shieldMax` (0 hides the shield bar), `segment` (shield per cell), `low` (a fraction of most health) | none |
| `ShooterSquad` | `members` keyed `{ id, name, portrait, status, state (ok, down, reviving, out), shield, shieldMax, health, healthMax, ghost }`, `segment`; a phone shows compact plates | none |
| `ShooterObjectives` | `label` (empty: Objective), `goals` keyed `{ id, text, tag, current }`; a phone shows the one in play alone | none |
| `ShooterRadar` | `map` (texture), `centerX`, `centerY`, `zoom`, `heading` (degrees), `blips` keyed `{ id, kind (ally, foe, objective), x, y, label }` | none |
| `ShooterRound` | `round` (0 hides it), `clock`, `ours`, `theirs` (negative hides the score), `urgent` | none |
| `ShooterPickups` | `items` keyed `{ id, name, icon, glyph, quantity, rarity (common, rare, epic, legendary) }`, newest first; a phone shows the newest | none |
| `ShooterDamage` | `hits` keyed `{ id, amount, kind (normal, headshot, shield, kill) }`, newest first | none |
| `ShooterPrompt` | `key`, `text` (empty hides it), `disabled`, `reason` | none |
| `ShooterEffects` | `effects` keyed `{ id, name, icon, glyph, remaining 0–1 (negative: no timer), label, harmful }` | none |
| `ShooterAbilities` | `slots` keyed `{ id, name, icon, glyph, key, cooldown 0–1, time, count, ready, disabled, ultimate, charge 0–1 }` | none |
| `CombatHud` | the members' inputs, plus `health`, `healthMax`, `shield`, `shieldMax` over core `StatBar`, and `feed` for the killfeed | none |

A locked slot is disabled and sends nothing. The Equip button is disabled
for a locked or equipped slot; X or pad Y on an equipped slot still sends
`equip`, and the game, which owns legality, ignores it. An empty loadout
says so and keeps Back. On a landscape phone (height under 600 px) the
title is smaller, the slots take one column and Back and Equip stand
beside the list at its foot, so the list keeps the page's height.
Equipped is a tag on a slot's detail line, so every row's price is one
right-aligned column. The slot list keeps a gutter for its scrollbar and
fades its bottom edge while there is more to scroll. Under a coarse pointer
the Back and Equip buttons drop their keycaps.

## Parts (dressed by packs)

| Part | What it is | Qualifiers |
| --- | --- | --- |
| `CombatHud.vitals` | The plate behind the health and shield bar | |
| `Crosshair.arm` | Each of the cross's four arms | `.aimed` |
| `Crosshair.ring` | The circle shape's ring: its ink is its `background` (a masked ring) | `.aimed` |
| `Crosshair.dot` | The centre dot | `.aimed` |
| `Crosshair.hit` | Each of the four hit ticks | `.headshot`, `.kill`, `.shieldBreak` |
| `Ammo.frame` | The readout (bare: it stands over the scene) | `.low`, `.empty` |
| `Ammo.alternate` | The reserve weapon's line | |
| `Ammo.swap` | The reserve weapon's key cap | |
| `Ammo.count` | Rounds in the magazine | `.low`, `.empty` |
| `Ammo.reserve` | Rounds in reserve | |
| `Ammo.rounds` | The magazine's ticks, one per round (the spent rounds' ink) | `.belt` |
| `Ammo.magazine` | The loaded rounds' ticks | |
| `Ammo.modes` | The fire mode's three pips | |
| `Ammo.weapon` | The weapon's name | |
| `Ammo.mode` | The fire mode in words | |
| `Ammo.reload` | The reload bar's fill | |
| `Ammo.prompt` | The empty magazine's prompt | |
| `DamageDirection.arc` | One incoming hit's arc (an SVG path: its ink is `stroke`) | `.shield`, `.critical` |
| `Killfeed.row` | One elimination | `.self` |
| `Killfeed.name` | A player's name | `.killer`, `.victim`, `.ally`, `.enemy` |
| `Killfeed.glyph` | The weapon's glyph, an SVG (its ink is `fill`) | |
| `Killfeed.weapon` | The weapon's name, where it has no glyph | |
| `Killfeed.headshot` | The headshot tag (HS) | |
| `Killfeed.edge` | The row's edge at the screen's side, lit for the player's rows | |
| `Loadout.category` | A category's heading | |
| `Loadout.item` | A slot, a button: hover, pressed, focused, selected and disabled come from the control | `.equipped`, `.locked` |
| `Loadout.detail` | A slot's detail line | |
| `Loadout.price` | A slot's cost | |

The `Shooter*` members name each piece a pack dresses in their `parts`
line (`ShooterVitals.portrait`, `.callsign`, `.shield`, `.health`,
`.ghost`; `ShooterAbilities.slot`, `.cooldown`, `.ring`, `.key`; and so
on); their look is the default style's `--hud-*` roles, so a pack that
sets those restyles the whole HUD.

A pack inks the text parts with `color`: the killfeed's names by team
(`[part="Killfeed.name"].ally`, `.enemy`), and the weapon, reserve, fire
mode and a slot's figures a step quieter. Text is sized in `em` steps
(`--font-step-*`) of the text around it, so it follows the player's text
scale; a pack that resizes a text part uses the same steps.
Undressed, they take the scene's or the menu's ink. The low and empty
count keeps the `--warn` and `--danger` tokens, since it is a signal
every pack must show. A `Killfeed.row` plate stays at
about 97 % opacity or more: rows sit over open sky, and the compositor
blends in linear light, so a lighter plate drops a team name under 4.5:1.

## Pages and frames (§6.2)

`og ui sheet shooter` renders `packages/engine/sheets/shooter/`. Its pages
use the components by name, as a game does.

- Normal: `combat` is the combat HUD over a bright sky and a dark ground,
  with a headshot mark and two incoming hits.
- Stress: `loadout` is a loadout with a long name, locked and equipped
  slots and more slots than a phone shows, beside a six-row killfeed with
  long names. The player is Kestrel: the newest row (Kestrel's kill) and the
  oldest (Kestrel's death) are self rows, framed; a phone shows only the
  newest of them.
- Motion: the sheet's commands `hit:<kind>`, `damage:<id>:<angle>:<strength>:<kind>`,
  `heal:<id>` and `reduced:<on|off>` drive the hit mark and the arcs for
  `og render --keyframes`. The mark pops in (opacity, and scale from 1.5)
  and holds while the game holds `hit`; an arc fades in and out over
  300 ms. Under reduced motion both only fade, and both settle: the frame's
  `dump_ui:motion` lists nothing moving once they have.
- Locales: the sheet loads the components' strings in Japanese, Simplified
  Chinese and Arabic (`assets/strings/`), so `og ui review shooter
  --locales game,pseudo,ja,ar` shows CJK and right to left. Right to left
  the HUD's corners mirror (the feed and ammunition on the left, the vitals
  on the right); the crosshair and the damage arcs keep their screen
  directions. The slot list's scrollbar stays on the right, with its room.
- Styles: the sheet's screens wear the `fantasy` pack and the default
  style (`combat-default`, `loadout-default`). A pack joins with a look
  naming it under `assets/looks/` (which makes the cook bundle it) and its
  name in the sheet's `STYLES`.

## Tests

`tests/shooter.test.ts` mounts every member from named stores. It covers
intents by row key from touch, keyboard and pad, and checks that disabled
slots send nothing. It covers the empty outcome and checks that selection
keeps its slot through a reorder, a removal and a restore. It also covers
hit-mark replay, the ammunition states, the order of keyed collections,
the safe area and spacing of the HUD on a desktop and on a phone both ways
(where the feed shows the newest row upright and three on its side), in the default style and in a pack, and
that the HUD's and the loadout's text follows the player's text scale.

## Document layout

The components use HTML with CSS, with `:host` styling the root and
conditional content built with `{#if}`. Ammunition and reload fills belong
to their own rounded tracks; the reticle and directional arcs own their
positioned layers. The scene halo inherits from the HUD root. Local-player
killfeed rows have a primary edge and extra breathing room, and loadout
categories and items declare their columns explicitly. Its footer never
shrinks into the phone's home area. Existing ids, parts and intents stay.

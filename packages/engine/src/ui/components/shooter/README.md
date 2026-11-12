# shooter: built-in components

Aim, ammunition, damage direction, killfeed and loadout, as documents over
core controls (docs/ui.md §4). They draw what the game tells it and
returns intents with stable ids. It never computes damage, spread,
reloads, kills or legality. Team and match scores belong to `arena`.

They are built into the engine: a game's page uses one by name
(`<Killfeed .../>`) and loading the page loads it, with the `StatBar`
that `CombatHud` composes. `og ui eject shooter/killfeed` copies one into
`assets/ui/shooter/` to change it; the game's copy of the same name wins.

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
| `AmmoCounter` | `weapon`, `mode` (auto, burst, single), `magazine`, `capacity`, `reserve`, `reloading` 0–1 (negative: not reloading), `low` (a fraction of capacity) | none |
| `DamageDirection` | `hits` keyed `{ id, angle (rad, 0 ahead, clockwise), strength 0–1, kind (normal, shield, critical) }` | none |
| `Killfeed` | `rows` keyed `{ id, killer, victim, weapon, headshot, self, team (ally or enemy, the killer's side) }`, newest first; a phone shows the three newest | none |
| `Loadout` | `title` (empty: the component's Loadout), `categories` keyed `{ id, label }` (an empty label: the component's string for primary, secondary, melee or utility), `slots` keyed `{ id, category, name, icon, detail, equipped, locked, cost }`, `selected` (a slot id) | `select` (row key: a tap, Enter or A on a slot), `equip` (row key: the Equip button sends the selected slot's, X or pad Y the focused slot's), `back` (the Back button, Escape or B) |
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

`Crosshair.arm`, `Crosshair.dot`, `Crosshair.hit[headshot|kill|shieldBreak]`
(the arms and dot take `.aimed`), `Ammo.frame`, `Ammo.count[low|empty]`,
`Ammo.reserve`, `Ammo.mode`, `Ammo.reload`,
`DamageDirection.arc[shield|critical]`, `Killfeed.row[self]`,
`Killfeed.name[killer|victim][ally|enemy]`, `Killfeed.weapon`,
`Loadout.item[equipped][locked]` (a button, so hover, pressed, focused,
selected and disabled come from the control), `Loadout.category`,
`Loadout.detail` and `Loadout.price`. A pack inks the text parts with
`-og-ink`: the killfeed's names by team (`Killfeed::name.ally`, `.enemy`),
and the weapon, reserve, fire mode and a slot's figures a step quieter.
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
- The primary pack is `clean`. The contrast pack, `scifi`, is pending
  because scifi is unbuilt.

## Tests

`tests/shooter.test.ts` mounts every member from named stores. It covers
intents by row key from touch, keyboard and pad, and checks that disabled
slots send nothing. It covers the empty outcome and checks that selection
keeps its slot through a reorder, a removal and a restore. It also covers
hit-mark replay, the ammunition states, the order of keyed collections,
and the safe area and spacing of the HUD on a desktop and a phone.

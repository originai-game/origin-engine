# Concepts

Targets for the art bar (docs/ui.md §15.5), made 2026-11-02 by the art
review with Painter, each from the current frame as its content reference.
Directions: A dark fantasy ARPG, B clean sci-fi, C bright stylised anime.
Screens: hud, menu, gacha, shop, inventory, settings, dialogue.

## Approved (owner, 2026-11-02)

| Direction | Becomes | Approved for |
| --- | --- | --- |
| B, clean sci-fi | the engine's default style and the UI kit's own frames | every screen: `B-hud`, `B-menu`, `B-gacha`, `B-shop`, `B-inventory`, `B-settings`, `B-dialogue` |
| A, dark fantasy | the fantasy seed style (`templates/styles/fantasy`) | every screen: `A-hud` … `A-dialogue` |
| C, bright stylised anime | the anime seed style | every screen: `C-hud` … `C-dialogue` |

- The kit's art stays painted (`templates/ui-kit/assets/textures/art`,
  docs/ui.md §15.5.3), never line icons or flat vector stand-ins.
- The showcase frames (the gallery, the store and docs images) wear the
  seed styles, with the default style as one column beside them.
- B sets the default's hierarchy, material, frames and art; its colour
  values are the design language's (`packages/design/tokens.json`).
- The default's primary is neutral (owner, 2026-11-02): neither the
  concept's cyan nor amber. B is drawn with neutral chrome (the primary
  button, selection, focus, tabs, sliders and rings told apart by tone,
  lift and weight), and hues appear only on semantic roles (success,
  warning, danger, info) and game roles (health, mana, xp, gold, rarity,
  team). Each game colours itself through its style; the seeds keep their
  own palettes. Hued chrome in a default frame is a review finding.

A frame is read side by side with the approved concept of its style's
direction and screen (docs/ui.md §15.5). A screen with no concept of its
own is read against the nearest one; the genre sheets below extend the
set.

## Genre sheets in B (2026-11-02)

The approved concepts are RPG screens; these extend the default style's
targets to every genre (owner, 2026-11-02). Each genre has its HUD and its
signature screen, drawn in B with the approved B sheets as the style
reference (`genres/B-<genre>-<screen>.jpg`). Status: drafts for the
owner; until one is approved, a frame of that genre is read against it as
the nearest concept.

Each row names the components that implement the screen today: U6c's HUD
rows (`packages/engine/src/ui/hud`, `world`, `feedback`), U6b's containers
and data, U6d's patterns (`ui/patterns`, on u6d-patterns-screens) and the
archetypes (`templates/archetypes`). **Gap** marks a part with no component
yet; the coordinator assigned each gap (2026-11-02), numbered as in its
brief: U6d 1 loadout and gunsmith, 2 garage, 4 deck builder, 5 saga map,
6 star-threshold bar and goal chips, 8 idle generator row and prestige,
9 VN box and choices, 11 song select; U6c 3 race standings and splits
(with the spline minimap and rear view), 7 RTS selection panel, command
card, production queue and box select, 10 sports timeline and player of
the match, 12 the small HUD items.

| Genre | Sheet | Implemented by | Gaps | Gap owner |
| --- | --- | --- | --- | --- |
| Shooter | `B-shooter-hud` | `crosshair`, `ammoCounter`, `killfeed`, `damageDirection`, `scorebug` and `hudTimer` (team scores and clock), `compass`, `minimapView`, `statBar` (health, armour), `abilitySlot` (grenade, gadget, the ultimate's ring), `damageNumbers` | objective diamonds on the compass row (A/B/C with team state), the operator portrait in `unitFrame`'s compact form | U6c (12) |
| Shooter | `B-shooter-loadout` | `tabBar` (loadouts), `statCompare` (stats with deltas), `inventoryGrid` rows | **Gap: loadout and gunsmith pattern**: slot list with art, a 3D weapon viewport (§5.5) with attachment sockets and leader lines, attachment list with rarity | U6d (1) |
| Racing | `B-racing-hud` | `speedometer` (speed, gear, revs), `statBar` segmented (nitro), `minimapView` | **Gap: race standings tower** (position, gap, car chip, the player's row), **gap: lap and split block** with signed deltas, track map as a spline outline (a `minimapView` mode), rear-view mirror (a render-to-texture view in the UI) | U6c (3) |
| Racing | `B-racing-garage` | `statCompare` (current and preview bars), badges (PI class), `tabBar` vertical | **Gap: garage pattern** (U6d's list): turntable viewport, category rail, part cards with tier pips, price and lock | U6d (2) |
| Card | `B-cards-combat` | `cardHand`, `unitFrame` with intent, `radialMeter` (energy), `statusGrid`, pile counters in `cardHand` | relic row (an icon strip with tooltips; `statusGrid` without timers may serve) | U6c (12) |
| Card | `B-cards-deck` | `inventoryGrid` (as a card grid), `chart` (mana curve), `tabBar`, search and segmented filters (U6a) | **Gap: deck builder and collection pattern**: owned counts, unowned dimmed, deck list rows with cost gem and art strip, the 30/30 counter | U6d (4) |
| Match-3 | `B-puzzle-board` | `radialMeter` and `numberCounter` (moves), `objectiveTracker` (goals), `abilitySlot` with charges (boosters), `match3` archetype (the board) | **Gap: star-threshold progress bar** (`statBar` with markers that fill), goal chips with item art | U6d (6) |
| Match-3 | `B-puzzle-map` | currency `numberCounter` with add buttons, banners (event card), primary button | **Gap: saga map pattern**: a scrolling painted map with a path of nodes, stars per node, the current node with the avatar pin, locked and boss nodes (the kit's `levels` screen is a grid) | U6d (5) |
| RTS | `B-rts-hud` | `minimapView` (fog, camera box, pings), `resourceBar`, `overheadBars`, `selectionRing`, `tabBar` (control groups) | **Gap: selection panel** (unit portrait grid with health, the primary unit's card), **gap: command card** (a hotkeyed grid of `abilitySlot`s), **gap: production queue**, drag-box select | U6c (7) |
| Tower defence | `B-td-build` | `radialMenu` (build options with cost and disabled), `tooltips` with `keyValueRows`, `telegraph` and `placementGhost` (range), `hudTimer` (wave), `resourceBar` (lives, gold), segmented speed control, `tower-defense` archetype | wave preview strip (`turnOrder` vertical may serve; to confirm), cost on `radialMenu` items turning red when unaffordable | U6c (12) |
| Idle | `B-idle-main` | `numberCounter` with notation, `damageNumbers` (floating gains), `statBar`, segmented buy multiplier, `idle` archetype | **Gap: generator row** (art, level, owned, cycle bar, rate, buy button with cost), **gap: prestige panel**, upgrade tiles (`inventoryGrid` may serve) | U6d (8) |
| Visual novel | `B-vn-dialogue` | none (`subtitles` is not a VN box) | **Gap: VN text box** (U6d's list): name plate, typewriter, advance mark, auto/skip/log/save/load row, backlog | U6d (9) |
| Visual novel | `B-vn-choice` | `hudTimer` (choice timer) | **Gap: choice list** with locked conditions ("Requires Trust 3") and a hovered lift | U6d (9) |
| Sports | `B-sports-hud` | `scorebug`, `minimapView` (pitch radar), `statBar` (stamina), banners (lower third), `statCompare` (possession) | player plate (compact `unitFrame` with number) | U6c (10) |
| Sports | `B-sports-scoreboard` | `scoreboard`, `statCompare` (if it draws centre-out two-sided bars; else a gap), results screen | **Gap: goal timeline** (scorers and minutes per side), player-of-the-match card (a `rewardChoice` card may serve) | U6c (10) |
| Rhythm | `B-rhythm-hud` | `judgement`, `comboCounter`, `numberCounter` (score), `statBar` (progress, fever), `rhythm` archetype (the highway) | lane keycaps on the hit line (`actionHints` glyphs at lane size) | U6c (12) |
| Rhythm | `B-rhythm-select` | `carousel` (vertical), segmented (difficulty), `radialMeter` (accuracy), badges (grade, full combo) | **Gap: song select pattern** (U6d's list): an angled wheel, jacket, preview waveform with play, difficulty chips with levels | U6d (11) |

## Seed styles the genres need (proposals, 2026-11-02)

A and C cover fantasy and anime. The genres above need four more looks,
each already a style pack under `templates/styles` that would become a
seed with an approved concept; one sample screen each in `seeds/`, for the
owner to choose:

| Proposal | Pack | For | Sample |
| --- | --- | --- | --- |
| Casual cartoon | `cartoon` (or `bubbly`) | match-3, idle, casual, party, runner | `seeds/casual-puzzle-map.jpg` |
| Tactical military | `tactical` | shooter, battle royale, tactics | `seeds/tactical-shooter-hud.jpg` |
| Retro pixel | `pixel` | platformer, roguelike, indie RPG | `seeds/pixel-hud.jpg` |
| Sports broadcast | `broadcast` | sports, racing, party results | `seeds/broadcast-scoreboard.jpg` |

## Every pack a seed (owner, 2026-11-02)

The owner's answer on the seed proposals: "cover the styles as completely
as possible". All four proposals are approved, and every style pack under
`templates/styles` becomes a seed with an approved concept sheet: three
screens, its genre's HUD, a menu or shop, and settings
(`packs/<pack>-{hud,menu,settings}.jpg`). A and C stay the fantasy and
kawaii (anime) sheets. B stays the default style's; the `scifi` pack has
its own harder sheet (hairlines, brackets, scanlines) so the two read
apart.

### Coverage

**P** is a genre's primary seed, S a secondary one. Every pack is primary
or secondary somewhere; the last column names the genres no pack carries
well and the new pack proposed for each.

| Genre | arcade | broadcast | bubbly | cartoon | craft | cyber | dread | fantasy | gilded | holo | inkwash | kawaii | minimal | neon | paper | pixel | scifi | storybook | tactical | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RPG and ARPG |  |  |  |  |  |  | S | **P** | S |  |  |  |  |  |  |  |  |  |  |  |
| Gacha |  |  |  |  |  |  |  |  | S | S |  | **P** |  |  |  |  |  |  |  |  |
| Shooter and battle royale |  |  |  |  |  | S |  |  |  |  |  |  |  |  |  |  | S |  | **P** |  |
| Racing | S | S |  |  |  |  |  |  |  |  |  |  |  | **P** |  |  |  |  |  |  |
| Cards and deckbuilder |  |  |  |  |  |  |  | S | **P** |  |  |  |  |  |  |  |  | S |  |  |
| Match-3 and casual puzzle |  |  | S | **P** |  |  |  |  |  |  |  | S |  |  |  |  |  |  |  |  |
| Saga (level-map meta) |  |  | **P** | S |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| Idle and incremental |  |  |  | **P** |  |  |  |  |  | S |  |  | S |  |  |  |  |  |  |  |
| RTS |  |  |  |  |  |  |  |  | S |  |  |  |  |  |  |  | **P** |  | S | foundry (industrial), midcore (mobile) |
| Tower defence |  |  |  | **P** |  |  |  | S |  |  |  |  |  |  |  |  | S |  |  | midcore (mobile) |
| Tactics |  |  |  |  |  |  |  | S |  | S |  |  |  |  |  |  |  |  | **P** |  |
| Roguelike |  |  |  |  |  |  | S |  |  |  |  |  |  |  |  | **P** |  | S |  |  |
| Platformer | S |  |  |  |  |  |  |  |  |  |  |  |  |  |  | **P** |  | S |  |  |
| Survival and crafting |  |  |  |  | **P** |  | S |  |  |  |  |  |  |  |  |  |  |  |  | **wasteland**: craft is cosy and dread is horror; neither is grunge survival |
| Farming and cozy |  |  |  |  | **P** |  |  |  |  |  |  |  |  |  | S |  |  | S |  |  |
| Horror |  |  |  |  |  |  | **P** |  |  |  |  |  |  |  | S |  |  |  |  |  |
| VN and romance |  |  |  |  |  |  |  |  |  |  | S | **P** |  |  | S |  |  |  |  |  |
| Sports | S | **P** |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| Rhythm |  |  |  |  |  | S |  |  |  |  |  | S |  | **P** |  |  |  |  |  |  |
| Party | S |  | **P** | S |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| Casino and board |  |  |  |  |  |  |  |  | **P** |  |  |  | S |  | S |  |  |  |  |  |
| MMO and raid |  |  |  |  |  |  | S | **P** | S |  |  |  |  |  |  |  |  |  |  |  |
| Kids and education |  |  | **P** | S |  |  |  |  |  |  |  |  |  |  |  |  |  | S |  |  |
| Wuxia and xianxia |  |  |  |  |  |  |  |  | S |  | **P** |  |  |  |  |  |  |  |  | **jade**: inkwash is restrained monochrome ink; xianxia wants carved jade and gold ornament |

### New packs for the gaps (proposals, one sample each in `proposals/`)

| Proposal | For | Why no pack carries it | Sample |
| --- | --- | --- | --- |
| `wasteland`: rusted plate, duct tape, stencils, warning orange, radiation green | survival and crafting, extraction shooters, post-apocalyptic RPG | craft is cosy, dread is gothic horror, tactical is clean military | `proposals/wasteland-hud.jpg` |
| `foundry`: dark iron, brass, rivets, gauges, furnace orange | city builders, factory and management sims, industrial RTS, 4X | gilded is luxury, fantasy is parchment; nothing is mechanical | `proposals/foundry-build.jpg` |
| `midcore`: thick outlines, bevelled stone and wood, heavy stroked type, glossy 3D icons | mobile strategy, auto-battler, tower defence, base building | cartoon and bubbly read soft and casual; this is competitive and chunky | `proposals/midcore-battle.jpg` (original units; a first draft that copied another game's characters is kept out of the repository) |
| `jade`: carved jade panels, gold cloud filigree, calligraphic titles, seal stamps | wuxia and xianxia, eastern fantasy gacha and MMO | inkwash is restrained monochrome ink; gilded is western | `proposals/jade-cultivation.jpg` |

### Sheets

Batches go to the coordinator for the owner's veto; a sheet not vetoed is
approved. Order is the style-art pipeline's: fantasy, kawaii, scifi,
cartoon, tactical, pixel, broadcast, then the rest by genre reach.

| Batch | Pack | HUD | Menu or shop | Settings | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | fantasy | `A-hud` | `A-shop` | `A-settings` | approved (A) |
| 1 | kawaii | `C-hud` | `C-shop` | `C-settings` | approved (C) |
| 1 | scifi | `packs/scifi-hud` (mech cockpit) | `packs/scifi-menu` (requisitions) | `packs/scifi-settings` | sent for veto |
| 1 | cartoon | `packs/cartoon-hud` (match-3 board) | `packs/cartoon-menu` (saga map) | `packs/cartoon-settings` | sent for veto |
| 1 | tactical | `packs/tactical-hud` (shooter) | `packs/tactical-menu` (loadout) | `packs/tactical-settings` | sent for veto |
| 1 | pixel | `packs/pixel-hud` (ability bar) | `packs/pixel-menu` (main menu) | `packs/pixel-settings` | sent for veto |
| 1 | broadcast | `packs/broadcast-hud` (football) | `packs/broadcast-menu` (full time) | `packs/broadcast-settings` | sent for veto |

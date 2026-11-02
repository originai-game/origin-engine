# Concepts

The images live in the showcase archive, not in git (docs/ci.md §Showcase):
`ui-art/<id>` names an entry of `showcase/manifest.jsonl`, and
`bun scripts/showcase.ts list --feature ui-art` lists them (`serve` shows
them). The first sheets (`A-*`, `B-*`, `C-*`) stay in this directory.

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

The gacha concepts are read in their versions with invented terms
(Summon, Limited banner, Forged Oath, Open Road): `A-gacha-v2` (`ui-art/4c03de42aadb`), `B-gacha-v2` (`ui-art/786ba2e42098`), `C-gacha-v2` (`ui-art/a120f7d09058`); the
first versions used another game's words.

A frame is read side by side with the approved concept of its style's
direction and screen (docs/ui.md §15.5). A screen with no concept of its
own is read against the nearest one; the genre sheets below extend the
set.

## Genre sheets in B (2026-11-02)

The approved concepts are RPG screens; these extend the default style's
targets to every genre (owner, 2026-11-02). Each genre has its HUD and its
signature screen, drawn in B with the approved B sheets as the style
reference (archived as `B-<genre>-<screen>`). Status: drafts for the
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
| Shooter | `B-shooter-hud` (`ui-art/952caf0e9a0e`) | `crosshair`, `ammoCounter`, `killfeed`, `damageDirection`, `scorebug` and `hudTimer` (team scores and clock), `compass`, `minimapView`, `statBar` (health, armour), `abilitySlot` (grenade, gadget, the ultimate's ring), `damageNumbers` | objective diamonds on the compass row (A/B/C with team state), the operator portrait in `unitFrame`'s compact form | U6c (12) |
| Shooter | `B-shooter-loadout` (`ui-art/65d978ed6e84`) | `tabBar` (loadouts), `statCompare` (stats with deltas), `inventoryGrid` rows | **Gap: loadout and gunsmith pattern**: slot list with art, a 3D weapon viewport (§5.5) with attachment sockets and leader lines, attachment list with rarity | U6d (1) |
| Racing | `B-racing-hud` (`ui-art/c719eebe1ccb`) | `speedometer` (speed, gear, revs), `statBar` segmented (nitro), `minimapView` | **Gap: race standings tower** (position, gap, car chip, the player's row), **gap: lap and split block** with signed deltas, track map as a spline outline (a `minimapView` mode), rear-view mirror (a render-to-texture view in the UI) | U6c (3) |
| Racing | `B-racing-garage` (`ui-art/f40a460c4546`) | `statCompare` (current and preview bars), badges (PI class), `tabBar` vertical | **Gap: garage pattern** (U6d's list): turntable viewport, category rail, part cards with tier pips, price and lock | U6d (2) |
| Card | `B-cards-combat` (`ui-art/cf3539994bb2`) | `cardHand`, `unitFrame` with intent, `radialMeter` (energy), `statusGrid`, pile counters in `cardHand` | relic row (an icon strip with tooltips; `statusGrid` without timers may serve) | U6c (12) |
| Card | `B-cards-deck` (`ui-art/ece2b469bc0a`) | `inventoryGrid` (as a card grid), `chart` (mana curve), `tabBar`, search and segmented filters (U6a) | **Gap: deck builder and collection pattern**: owned counts, unowned dimmed, deck list rows with cost gem and art strip, the 30/30 counter | U6d (4) |
| Match-3 | `B-puzzle-board` (`ui-art/9787af15b471`) | `radialMeter` and `numberCounter` (moves), `objectiveTracker` (goals), `abilitySlot` with charges (boosters), `match3` archetype (the board) | **Gap: star-threshold progress bar** (`statBar` with markers that fill), goal chips with item art | U6d (6) |
| Match-3 | `B-puzzle-map` (`ui-art/5192dee069ca`) | currency `numberCounter` with add buttons, banners (event card), primary button | **Gap: saga map pattern**: a scrolling painted map with a path of nodes, stars per node, the current node with the avatar pin, locked and boss nodes (the kit's `levels` screen is a grid) | U6d (5) |
| RTS | `B-rts-hud` (`ui-art/c81fdc034dc0`) | `minimapView` (fog, camera box, pings), `resourceBar`, `overheadBars`, `selectionRing`, `tabBar` (control groups) | **Gap: selection panel** (unit portrait grid with health, the primary unit's card), **gap: command card** (a hotkeyed grid of `abilitySlot`s), **gap: production queue**, drag-box select | U6c (7) |
| Tower defence | `B-td-build` (`ui-art/cd8369605f02`) | `radialMenu` (build options with cost and disabled), `tooltips` with `keyValueRows`, `telegraph` and `placementGhost` (range), `hudTimer` (wave), `resourceBar` (lives, gold), segmented speed control, `tower-defense` archetype | wave preview strip (`turnOrder` vertical may serve; to confirm), cost on `radialMenu` items turning red when unaffordable | U6c (12) |
| Idle | `B-idle-main` (`ui-art/4503bf90089d`) | `numberCounter` with notation, `damageNumbers` (floating gains), `statBar`, segmented buy multiplier, `idle` archetype | **Gap: generator row** (art, level, owned, cycle bar, rate, buy button with cost), **gap: prestige panel**, upgrade tiles (`inventoryGrid` may serve) | U6d (8) |
| Visual novel | `B-vn-dialogue` (`ui-art/439fb89beba0`) | none (`subtitles` is not a VN box) | **Gap: VN text box** (U6d's list): name plate, typewriter, advance mark, auto/skip/log/save/load row, backlog | U6d (9) |
| Visual novel | `B-vn-choice` (`ui-art/c92e209fb322`) | `hudTimer` (choice timer) | **Gap: choice list** with locked conditions ("Requires Trust 3") and a hovered lift | U6d (9) |
| Sports | `B-sports-hud` (`ui-art/c22af9a95269`) | `scorebug`, `minimapView` (pitch radar), `statBar` (stamina), banners (lower third), `statCompare` (possession) | player plate (compact `unitFrame` with number) | U6c (10) |
| Sports | `B-sports-scoreboard` (`ui-art/7e2b3d357e54`) | `scoreboard`, `statCompare` (if it draws centre-out two-sided bars; else a gap), results screen | **Gap: goal timeline** (scorers and minutes per side), player-of-the-match card (a `rewardChoice` card may serve) | U6c (10) |
| Rhythm | `B-rhythm-hud` (`ui-art/18cb9a534851`) | `judgement`, `comboCounter`, `numberCounter` (score), `statBar` (progress, fever), `rhythm` archetype (the highway) | lane keycaps on the hit line (`actionHints` glyphs at lane size) | U6c (12) |
| Rhythm | `B-rhythm-select` (`ui-art/9895b8d408d9`) | `carousel` (vertical), segmented (difficulty), `radialMeter` (accuracy), badges (grade, full combo) | **Gap: song select pattern** (U6d's list): an angled wheel, jacket, preview waveform with play, difficulty chips with levels | U6d (11) |

## Seed styles the genres need (proposals, 2026-11-02)

A and C cover fantasy and anime. The genres above need four more looks,
each already a style pack under `templates/styles` that would become a
seed with an approved concept; one sample screen each in the archive, for the
owner to choose:

| Proposal | Pack | For | Sample |
| --- | --- | --- | --- |
| Casual cartoon | `cartoon` (or `bubbly`) | match-3, idle, casual, party, runner | `casual-puzzle-map` (`ui-art/18417a3dbdad`) |
| Tactical military | `tactical` | shooter, battle royale, tactics | `tactical-shooter-hud` (`ui-art/5dc0d4592974`) |
| Retro pixel | `pixel` | platformer, roguelike, indie RPG | `pixel-hud` (`ui-art/3383564cc379`) |
| Sports broadcast | `broadcast` | sports, racing, party results | `broadcast-scoreboard` (`ui-art/734f4c4d2379`) |

## Every pack a seed (owner, 2026-11-02)

The owner's answer on the seed proposals: "cover the styles as completely
as possible". All four proposals are approved, and every style pack under
`templates/styles` becomes a seed with an approved concept sheet: three
screens, its genre's HUD, a menu or shop, and settings
(archived as `<pack>-{hud,menu,settings}`). A and C stay the fantasy and
kawaii (anime) sheets. B stays the default style's; the `scifi` pack has
its own harder sheet (hairlines, brackets, scanlines) so the two read
apart.

### Coverage

Sixteen packs (the owner, 2026-11-02): fantasy, jade, kawaii, cartoon, midcore, storybook, craft, pixel, gilded, tactical, wasteland, dread, scifi, cyber, foundry, broadcast. **P** is a
genre's primary seed, S a secondary one; every pack is primary or
secondary somewhere. Six packs were merged into others by the owner, and
the absorbing pack takes over their genres and their sheets' useful parts
(below): bubbly into cartoon, arcade into pixel, paper into storybook, inkwash into jade, neon into cyber, holo into scifi. `minimal` was
dropped. `scifi` is now real-world spaceflight operations and `cyber` the
high-contrast yellow, red and cyan look; both replace their first sheets.

| Genre | fantasy | jade | kawaii | cartoon | midcore | storybook | craft | pixel | gilded | tactical | wasteland | dread | scifi | cyber | foundry | broadcast |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RPG and ARPG | **P** |  |  |  |  |  |  |  | S |  |  | S |  |  |  |  |
| MMO and raid | **P** | S |  |  |  |  |  |  | S |  |  | S |  |  |  |  |
| Gacha |  | S | **P** |  |  |  |  |  | S |  |  |  |  |  |  |  |
| VN and romance |  | S | **P** |  |  | S |  |  |  |  |  |  |  |  |  |  |
| Wuxia and xianxia |  | **P** |  |  |  |  |  |  | S |  |  |  |  |  |  |  |
| Shooter and battle royale |  |  |  |  |  |  |  |  |  | **P** | S |  |  | S |  |  |
| Tactics | S |  |  |  |  |  |  |  |  | **P** |  |  | S |  |  |  |
| Racing |  |  |  |  |  |  |  | S |  |  |  |  |  | **P** |  | S |
| Rhythm |  |  | S |  |  |  |  |  |  |  |  |  |  | **P** |  |  |
| Sports |  |  |  |  |  |  |  | S |  |  |  |  |  |  |  | **P** |
| Cards and deckbuilder | S |  |  |  |  | S |  |  | **P** |  |  |  |  |  |  |  |
| Casino and board |  |  |  |  |  | S |  |  | **P** |  |  |  |  |  |  |  |
| Match-3 and casual puzzle |  |  | S | **P** |  |  |  |  |  |  |  |  |  |  |  |  |
| Saga (level-map meta) |  |  | S | **P** |  |  |  |  |  |  |  |  |  |  |  |  |
| Idle and incremental |  |  |  | **P** |  |  |  |  |  |  |  |  |  |  | S |  |
| Party |  |  |  | **P** |  |  |  | S |  |  |  |  |  |  |  |  |
| Kids and education |  |  |  | **P** |  | S |  |  |  |  |  |  |  |  |  |  |
| Tower defence | S |  |  | **P** | S |  |  |  |  |  |  |  |  |  |  |  |
| Mobile midcore and auto-battler |  |  |  | S | **P** |  |  |  |  |  |  |  |  |  |  |  |
| RTS |  |  |  |  | S |  |  |  |  | S |  |  | **P** |  | S |  |
| Space and flight sim |  |  |  |  |  |  |  |  |  | S |  |  | **P** |  |  |  |
| City builder, factory and 4X |  |  |  |  |  |  | S |  |  |  |  |  |  |  | **P** |  |
| Survival, crafting and extraction |  |  |  |  |  |  | S |  |  |  | **P** | S |  |  |  |  |
| Farming and cozy |  |  |  |  |  | S | **P** |  |  |  |  |  |  |  |  |  |
| Horror |  |  |  |  |  | S |  |  |  |  |  | **P** |  |  |  |  |
| Roguelike |  |  |  |  |  | S |  | **P** |  |  |  | S |  |  |  |  |
| Platformer |  |  |  |  |  | S |  | **P** |  |  |  |  |  |  |  |  |

### What each absorbing pack takes

| Pack | Absorbs | Takes from the absorbed sheet |
| --- | --- | --- |
| cartoon | bubbly | the party HUD (player-colour score chips, round timer, round results), the store's bundle cards with value ribbons and glossy currency pills, the squash-and-stretch press on primary buttons |
| pixel | arcade | the high-score table (rank, initials, zero-padded score, metal medals for 1–3, initials entry with a blinking cursor), score and HI counters, hearts and lives, coin and timer counters, the power-up slot, segmented pixel sliders, attract-mode text |
| storybook | paper | the visual-novel set: dialogue box with a name tab and continue mark, the Log, Save, Load, Auto and Skip strip, a clue notebook tab, the text options (speed, auto-play delay, window opacity, readable-font toggle) with a live preview card, index-card menu items; paper cut-out layering as storybook's lighter variant |
| jade | inkwash | restrained monochrome ink-wash backdrops and brush-stroke bars as jade's quiet variant, brush calligraphy display titles, the wuxia combat HUD's stance and qi readouts |
| cyber | neon | the racing HUD (speedometer with gear and revs, position and lap block, segmented nitro bar) and the rhythm song select (song list with art, difficulty, BPM, best score), redrawn in cyber's palette and cut corners, not neon's magenta synthwave |
| scifi | holo | the starship tactics parts (ship status with hull, shields and reactor, turn order, ability cards, sector map) and the fleet hangar, redrawn in the spaceflight grammar: flat near-black glass, thin white sans, one ice-blue accent; plus the RTS selection panel and command card from the first scifi sheet, in the same grammar |

### New packs for the gaps (approved by the owner, 2026-11-02)

Approved by the coordinator under the owner's "cover the styles as
completely as possible" (2026-11-02). Each becomes a pack after batch 3,
with a sheet of its own, and is the primary seed of: `wasteland` survival
and extraction, `foundry` city builders, factory and 4X, `midcore` mobile
midcore and auto-battlers, `jade` xianxia.

| Proposal | For | Why no pack carries it | Sample |
| --- | --- | --- | --- |
| `wasteland`: rusted plate, duct tape, stencils, warning orange, radiation green | survival and crafting, extraction shooters, post-apocalyptic RPG | craft is cosy, dread is gothic horror, tactical is clean military | `wasteland-hud` (`ui-art/13876c1858fa`) |
| `foundry`: dark iron, brass, rivets, gauges, furnace orange | city builders, factory and management sims, industrial RTS, 4X | gilded is luxury, fantasy is parchment; nothing is mechanical | `foundry-build` (`ui-art/a19f7660818a`) |
| `midcore`: thick outlines, bevelled stone and wood, heavy stroked type, glossy 3D icons | mobile strategy, auto-battler, tower defence, base building | cartoon and bubbly read soft and casual; this is competitive and chunky | `midcore-battle` (`ui-art/b88efa4bb6ac`) (original units; a first draft that copied another game's characters is kept out of the repository) |
| `jade`: carved jade panels, gold cloud filigree, calligraphic titles, seal stamps | wuxia and xianxia, eastern fantasy gacha and MMO | inkwash is restrained monochrome ink; gilded is western | `jade-cultivation` (`ui-art/dba0973d790c`) |

### Sheets

Batches go to the coordinator for the owner's veto; a sheet not vetoed is
approved. Order is the style-art pipeline's: fantasy, kawaii, scifi,
cartoon, tactical, pixel, broadcast, then the rest by genre reach.

| Batch | Pack | HUD | Menu or shop | Settings | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | fantasy | `A-hud` | `A-shop` | `A-settings` | approved (A) |
| 1 | kawaii | `C-hud` | `C-shop` | `C-settings` | approved (C) |
| 1 | scifi | `scifi-hud` (`ui-art/037fdb651dcd`) (mech cockpit) | `scifi-menu` (`ui-art/b0fc49921189`) (requisitions) | `scifi-settings` (`ui-art/96b1cf217878`) | superseded by the owner's redirection (below) |
| 1 | cartoon | `cartoon-hud` (`ui-art/b476bf510b9c`) (match-3 board) | `cartoon-menu` (`ui-art/18417a3dbdad`) (saga map) | `cartoon-settings` (`ui-art/3fd89c603e7c`) | approved (owner) |
| 1 | tactical | `tactical-hud` (`ui-art/5dc0d4592974`) (shooter) | `tactical-menu` (`ui-art/9a9110d75f5e`) (loadout) | `tactical-settings` (`ui-art/673672bc2ab1`) | approved (owner) |
| 1 | pixel | `pixel-hud` (`ui-art/3383564cc379`) (ability bar) | `pixel-menu` (`ui-art/187c8df6ca84`) (main menu) | `pixel-settings` (`ui-art/41124ea7b1be`) | approved (owner) |
| 1 | broadcast | `broadcast-hud` (`ui-art/342c9b49d2e6`) (football) | `broadcast-menu` (`ui-art/734f4c4d2379`) (full time) | `broadcast-settings` (`ui-art/e16be56d19f0`) | approved (owner) |
| 2 | gilded | `gilded-hud` (`ui-art/23b214a1d6ae`) (card combat) | `gilded-menu` (`ui-art/75f688ef5276`) (collection and deck) | `gilded-settings` (`ui-art/6ca093ebfc1e`) | approved (owner, no veto) |
| 2 | neon | `neon-hud` (`ui-art/fc3155f3505a`) (racing) | `neon-menu` (`ui-art/ad494d89ad97`) (song select) | `neon-settings` (`ui-art/c9aae1a6c66e`) | merged into cyber by the owner |
| 2 | bubbly | `bubbly-hud` (`ui-art/a944c75f8827`) (party) | `bubbly-menu` (`ui-art/fd559b5c997e`) (store) | `bubbly-settings` (`ui-art/cee7e332670b`) | merged into cartoon by the owner |
| 2 | dread | `dread-hud` (`ui-art/f3c187324b73`) (survival horror) | `dread-menu` (`ui-art/88d34c09a6b2`) (main menu) | `dread-settings` (`ui-art/94d363b1c999`) | approved (owner, no veto) |
| 2 | craft | `craft-hud` (`ui-art/ffd3635f6b19`) (farming) | `craft-menu` (`ui-art/a9e6efac42b5`) (crafting) | `craft-settings` (`ui-art/84ffbbb1286f`) | approved (owner, no veto) |
| 2 | inkwash | `inkwash-hud` (`ui-art/e3d0e58be15b`) (wuxia) | `inkwash-menu` (`ui-art/375e297c719f`) (main menu) | `inkwash-settings` (`ui-art/818e42ee2b42`) | merged into jade by the owner |
| 3 | holo | `holo-hud` (`ui-art/0cccb255873a`) (starship tactics) | `holo-menu` (`ui-art/11d3db9b1c45`) (fleet hangar) | `holo-settings` (`ui-art/b1df839a20f1`) | merged into scifi by the owner |
| 3 | cyber | `cyber-hud` (`ui-art/0bfbbbc90725`) (heist shooter) | `cyber-menu` (`ui-art/4fbdde732ece`) (cyberware market) | `cyber-settings` (`ui-art/0617c70348e6`) | superseded by the owner's redirection (below) |
| 3 | arcade | `arcade-hud` (`ui-art/324d90871abe`) (platformer) | `arcade-menu` (`ui-art/87bde4bbebde`) (title and high scores) | `arcade-settings` (`ui-art/1ea7aecafa41`) | merged into pixel by the owner |
| 3 | storybook | `storybook-hud` (`ui-art/dcc5fae146a1`) (card adventure) | `storybook-menu` (`ui-art/e181cf7311d3`) (merchant shop) | `storybook-settings` (`ui-art/58673c78f815`) | approved (owner) |
| 3 | paper | `paper-hud` (`ui-art/b05ba49c6229`) (visual novel) | `paper-menu` (`ui-art/b02bf53f590a`) (main menu) | `paper-settings` (`ui-art/4075524240e4`) (text options) | merged into storybook by the owner |
| 4 | wasteland | `wasteland-hud` (`ui-art/13876c1858fa`) (survival HUD) | `wasteland-menu` (`ui-art/37b06d223094`) (stash and crafting) | `wasteland-settings` (`ui-art/860fbeffcbb7`) | approved (owner) |
| 4 | foundry | `foundry-build` (`ui-art/a19f7660818a`) (city build) | `foundry-menu` (`ui-art/2593651e20db`) (research tree) | `foundry-settings` (`ui-art/2b3bf48492bf`) | approved (owner) |
| 4 | midcore | `midcore-battle` (`ui-art/b88efa4bb6ac`) (battle) | `midcore-menu` (`ui-art/1aa6c4f07057`) (hero upgrade) | `midcore-settings` (`ui-art/442a633f59e7`) | approved (owner) |
| 4 | jade | `jade-cultivation` (`ui-art/dba0973d790c`) (cultivation) | `jade-menu` (`ui-art/98481c251aeb`) (treasure pavilion) | `jade-settings` (`ui-art/3f19757c41e2`) | approved (owner) |
| 5 | scifi | `scifi-hud` (`ui-art/70c6e287e1a1`) (docking approach) | `scifi-menu` (`ui-art/371bdbc76e8d`) (mission planning) | `scifi-settings` (`ui-art/5c356aaad012`) | sent for the owner's look |
| 5 | cyber | `cyber-hud` (`ui-art/46b39cda6040`) (rooftop heist) | `cyber-menu` (`ui-art/b084a43067cb`) (implant dealer) | `cyber-settings` (`ui-art/3cdb44b05813`) | sent for the owner's look |

The holo and cyber thumbnail directions (`ui-art/62c2ccec6aac` … `ui-art/fab9821241f6`) were set aside by the owner's call for spaceflight operations (scifi) and the high-contrast cyber look.

# Built-in components

The engine's UI components, TSX functions styled with Tailwind classes
(`crates/og-ui-next/CONVENTIONS.md`), filed by what they do: `base/` (small
building blocks: slots, avatars, currency, spinners), `hud/` (bars, meters,
buttons, timers and feeds drawn over play), `world/` (tags anchored on
entities and maps), `panels/` (data, menus, inventories, quest logs,
conversation, charts and graphs), `meta/` (shop, social, rewards and
progression), and a genre's own pieces in its kit (`rpg/`, `shooter/`,
`strategy/`, `racing/`, `card/`, `puzzle/`, `idle/`, `rhythm/`, `arena/`).
A piece any genre uses is filed by function, not under the genre that first
needed it.

A game imports one by name from `@origingame/engine/ui/components`
(`index.tsx` exports them all) or by path
(`@origingame/engine/ui/components/hud/stat-bar`), and mounts it or calls it
inside its own component. To change one, copy its `.tsx` (and its string
table) into the game and import the copy.

## A component file

`<dir>/<name>.tsx` holds one component and what describes it:

- `export interface <Name>Props`: every prop with a doc comment saying what
  it is, its values and its default. A prop the game may leave out is
  optional and has its default in the body, so a component draws something
  sensible from `fixtures.default` alone.
- `export function <Name>(props)`: built from the primitives (`View`,
  `Text`, `Image`, `Icon`, `Scroll`, `Input`, `Line`) and `Show`, `For`,
  `Switch`. Its doc comment says what it draws and what it reports.
- `export const fixtures`: `default` and a named case for each state worth
  seeing (empty, overflow, locked, compact). The gallery and the tests draw
  every fixture, merged over `default`.
- `export const actions` (when it answers input actions): each action and
  the label a hint shows for it.
- Visible words come from its string table, imported beside it
  (`import "./stat-bar.en.ogt"`), through `t(key, params)` and the
  `format*` helpers; text is never concatenated from translated pieces.

It draws what the game hands it and reports intents through `on*` props
(`onChoose(e, id)`, `onPick(e, id)`) with stable row keys. It never decides
costs, legality, rewards or rules.

## Look

- **Classes only, theme roles only.** Colours, radii, shadows, type and
  spacing are Tailwind classes over the theme
  (CONVENTIONS.md §Theme and packs, "The default pack's names"). A literal
  colour class (`bg-[#…]`) is refused by `bun run typecheck`: name its role,
  or add the role to `packages/design/tokens/color.css`. An inline `style`
  carries only what a prop computes (a fill's width, a node's position).
- **Packs dress slots.** The surface a pack restyles ends its class with
  `packClass("Panel", "frame", variant)` (slots: `Button.plate`,
  `Panel.frame`, `Plate.frame`, `Slot.frame`, `Chip.frame`,
  `Field.frame`). A slot that paints its own fill also owns the ink on it.
- **Shapes and art are images.** Cut corners, sockets and frames are a
  pack's nine-slice art as `nine-<name>` classes: `plate-cut`, `panel-cut`,
  `chip-cut`, `tile`, `disc`, `ring` (tinted by `bg-*`), and every look's
  generated `socket`, `socket-rim` (tinted), `glow` (tinted), `sheen` and
  `card-frame`. Cooldown sweeps and rings are the `wipe` prop on such a box.
- **States are `data-*` booleans.** `data-done={…}` is on unless false or
  empty, read by `data-done:` on the element and
  `group-data-done/<group>:` below a `group/<group>`. A value is not
  matched: give each value its own attribute (`data-rarity-rare`,
  `data-rarity-epic`), never `data-rarity={"rare"}`.
- **Geometry comes from the game** as numbers: a `Line`'s `points` (x, y
  pairs, with `viewBox` for a space of its own), a chart's values, a graph's
  node positions, a radial menu's slice angles. Nothing parses SVG path
  data.
- **Compact** layouts are `compact:` classes (under 600 px wide or 500 px
  tall).

World markers on the ground (selection rings, telegraphs, placement
ghosts, paths) are not UI: they are og.Decal and og.render.lines
(`og api show effects`). Tags over entities are UI, with the `anchor` prop.

## Catalogue

`bun packages/engine/src/node/gallery.ts` renders every fixture of every
component under every pack; `--only hud/` keeps one directory, `--packs`
some packs. It exits non-zero and writes `defects.txt` for a fixture that
throws at mount, a raw string key drawn, or a word broken inside.

| Purpose | Components |
|---|---|
| Item slots (rarity, count, cooldown and selection) | `base/item-slot` |
| Player portraits (fallback, level and presence) | `base/avatar` |
| Read-only currency balances, capacity and income | `base/currency` |
| Discrete resource pips | `base/pips` |
| Empty and error states | `base/empty-state`, `base/error-state` |
| List rows and step progress | `base/list-item`, `base/steps` |
| Boss health, combo, pickup activity and performance readouts | `hud/boss-bar`, `hud/combo`, `hud/pickup-feed`, `hud/perf-readout` |
| Quick-time event prompts | `hud/qte-prompt` |
| Pass progression and reward reveals | `meta/pass-track`, `meta/reveal` |
| Inbox and news panels | `meta/inbox`, `meta/news` |
| Player profiles and game-owned actions | `meta/profile` |
| Crafting recipes, trade stock and loot collection | `panels/crafting`, `panels/trade`, `panels/loot` |
| Conversation history and replay | `panels/conversation-backlog` |
| Ability bars, cooldowns and buffs | `hud/ability-bar`, `hud/ability-slot`, `hud/cooldown`, `hud/status-grid` |
| Meters and resource bars | `hud/stat-bar`, `hud/orb-meter`, `hud/radial-meter` |
| Progress and loading placeholders | `hud/progress-bar`, `hud/progress-ring`, `base/skeleton-group`, `base/spinner` |
| Timers and scores | `hud/hud-timer`, `hud/star-bar` |
| Celebrations (confetti, a burst, a level up, fireworks) | `hud/celebration` |
| Navigation (minimap, compass) | `world/minimap`, `world/minimap-view`, `world/compass` |
| Menus and layout (radial menu, split view, carousel, breadcrumbs) | `hud/radial-menu`, `panels/split-view`, `panels/carousel`, `panels/breadcrumbs` |
| Menus and tooltip cards (in a `popover` or `contextmenu` popover) | `panels/menu`, `panels/tooltip-card` |
| Charts and map overlays | `panels/chart`, `hud/overlay-bar` |
| Skill trees and run maps | `panels/node-graph` |
| Data views (inventories, level selects) | `panels/data-grid` |
| Buttons and badges (hero button, touch action, hold to confirm, action bar, ribbon) | `hud/hero-button`, `hud/action-button`, `hud/hold-button`, `hud/action-hints`, `base/ribbon` |
| Touch controls (a stick or d-pad and the action buttons, shown under a finger) | `hud/touch-controls` |
| World tags over units and items (`world/README.md`) | `world/nameplate`, `world/waypoint`, `world/loot-label`, `world/speech-bubble`, `world/interaction-prompt` |
| Mass world elements (overhead bars, damage numbers) | `world/overhead-bars`, `hud/damage-numbers` |
| RPG inventory, character and equipment (`rpg/README.md`) | `panels/backpack`, `panels/item-card`, `rpg/character`, `rpg/stats`, `rpg/menu-bar` |
| RPG quests, world map and conversation | `panels/quest-log`, `rpg/quest-detail`, `hud/quest-tracker`, `rpg/world-map`, `rpg/place-card`, `panels/conversation` |
| RPG party, turn order and combat HUD | `rpg/vitals`, `rpg/party`, `rpg/turn-order`, `hud/combat-log`, `rpg/enemy-info`, `rpg/commands` |
| RPG action HUD: leader, effects, companions, minimap, lock-on, damage figures, combo, loot feed, prompt, skills, hints, experience | `rpg/leader`, `rpg/effects`, `rpg/companions`, `rpg/minimap`, `rpg/lock-on`, `rpg/hits`, `rpg/combo`, `rpg/loot-feed`, `rpg/prompt`, `rpg/skills`, `rpg/hints`, `rpg/experience` |
| HUD blocks, which every HUD family's plates and rows are built from and a game lays out itself: portrait, bar, slot, effect, hint, loot entry (`HudPortrait`, `HudBar`, `HudSlot`, `HudEffect`, `HudHint`, `HudLootEntry`) | `hud/portrait`, `hud/bar`, `hud/slot`, `hud/effect`, `hud/hint`, `hud/loot-entry` |
| Shooter HUD: aim, ammunition, damage direction, killfeed (`shooter/README.md`) | `hud/crosshair`, `hud/ammo-counter`, `hud/damage-direction`, `hud/killfeed`, `shooter/combat-hud` |
| Shooter HUD: squad, objectives, radar, round, pickups, damage dealt, prompt, effects, vitals, abilities | `shooter/squad`, `shooter/objectives`, `shooter/radar`, `shooter/round`, `shooter/pickups`, `shooter/damage`, `shooter/prompt`, `shooter/effects`, `shooter/vitals`, `shooter/abilities` |
| Shooter loadout | `shooter/loadout` |
| Strategy resources, clock, waves, alerts, control groups, minimap, selection, production queue, commands, tower upgrades (`strategy/README.md`) | `strategy/resources`, `strategy/clock`, `strategy/wave`, `strategy/alerts`, `strategy/groups`, `strategy/minimap`, `strategy/selection`, `strategy/queue`, `strategy/commands`, `strategy/tower` |
| Racing telemetry, laps, standings, banner, track map, boost, pedals, car, bonus, garage, results (`racing/README.md`) | `racing/telemetry`, `racing/lap-hud`, `racing/standings`, `racing/banner`, `racing/track-map`, `racing/boost`, `racing/pedals`, `racing/car`, `racing/bonus`, `racing/garage`, `racing/results` |
| Card hand, piles, collection, deck, rewards (`card/README.md`) | `card/face`, `card/hand`, `card/piles`, `card/collection`, `card/deck-list`, `card/reward` |
| Puzzle goals, boosters, level path, summary (`puzzle/README.md`) | `puzzle/goals`, `puzzle/boosters`, `puzzle/level-path`, `puzzle/summary` |
| Idle resources, generators, purchases, prestige, offline summary (`idle/README.md`) | `idle/resources`, `idle/generators`, `idle/purchases`, `idle/prestige`, `idle/offline` |
| Rhythm song and difficulty selection, score HUD, judgement, results (`rhythm/README.md`) | `rhythm/song-select`, `rhythm/difficulty`, `rhythm/song-plate`, `rhythm/song-progress`, `rhythm/chain`, `rhythm/score-hud`, `rhythm/gauge`, `rhythm/judgement`, `rhythm/timing`, `rhythm/lane-keys`, `rhythm/results` |
| Arena roster, draft, versus, lobby, round HUD, scoreboard (`arena/README.md`) | `arena/roster`, `arena/draft`, `arena/versus`, `arena/lobby`, `arena/round-hud`, `arena/scoreboard` |
| Arena ready checks | `arena/ready` |
| Arena map votes | `arena/vote` |
| Arena spectator controls | `arena/spectator` |
| Shop, achievements, daily quests and rewards, chat, friends (`meta/README.md`) | `meta/shop`, `meta/shop-confirm`, `meta/achievements`, `meta/daily-quests`, `meta/daily-rewards`, `meta/chat`, `meta/friends` |

A chip, a tag or a filter toggle has no component: it is a `View` with
`role="button"`, `pressed={on}` and `packClass("Chip", "frame")`, with a
`data-*` state for its look when on.

`hud/touch-controls` is the whole on-screen pad: a stick or d-pad at the
start side (`touchStick`, `touchPad`), action buttons at the end side
(`touchButton`) and an optional second stick, each feeding the og-input
action it names, so the game reads the same actions as from a pad. It
reports no events and shows only where the primary pointer is a finger.

Adding, renaming or removing a component adds or removes its line in
`index.tsx` in the same head.

# Component documents

Component documents built from og-ui's public elements and built into the
engine, filed by what they do: `base/` (small building blocks: slots,
avatars, currency, spinners), `hud/` (bars, meters, buttons, timers, feeds
drawn over play), `world/` (tags, ground marks and maps anchored in the
world), `panels/` (data, menus, inventories, quest logs, conversation),
`meta/` (shop, social, rewards and progression, with its README), and a
genre's own screens in its kit (`rpg/`, `shooter/`, `strategy/` and the
rest, each with its README). A piece any genre uses is filed by function,
not under the genre that first needed it. A game's document
uses any of them by its declared name (`<AbilityBar slots={$slots} />`,
`<RpgBackpack items={$items} />`) without a copy: the cook bundles it as
`@ui/components/<path>` and the screen that uses it loads it. Copy one with
`og ui eject <component>` (`og ui eject panels/backpack` in a folder) and edit
it in `assets/ui/<component>.svelte`; the game's copy then replaces the
built-in one of its name.

They follow the screen conventions (`../screens/README.md`): each declares
`<meta name="component">`, its props and actions, and a
`<meta name="fixture:default">` JSON fixture. Every prop has a default, so
`<StatBar />` draws an empty health bar and a game passes only what it
shows; structure is scriptless markup,
look is scoped CSS and tokens, rings and bars are inline SVG, world tags are
`<anchor>`, ground marks `<ground>`, and overhead bars and damage numbers are
`<anchor mass>` rows drawn on the GPU. Visible words come from the companion
string table (`<meta name="strings" content="./<component>.en.ogt">`), which
eject copies too.

HTML elements own layout through scoped CSS: component roots use `:host`,
optional pieces use `{#if}`, and positioned layers measure against their
own positioned well or track. Filled surfaces and typography use shared
default-style tokens; style packs restyle meaningful pieces through parts.
The virtualised `<list>` and `<cells>` elements remain until their separate
list conversion. World anchors, ground marks and mass rows remain primitives.

Expressions compute no geometry, so the game hands in what geometry a
component needs: a chart's line and a path line's route as SVG path data, a radial
menu's slice positions (`at`, `span`), a split view's ratio from its
divider's pointer events, a ground mark's world centre.

`og ui eject --list` prints these with the screens. The component document
test cooks every one from its fixture at desktop and phone sizes.

## Review pages

Every named page below has a `<page>-default` screen with
`style:default;page:<page>`; render desktop and `phonePortrait`. The
components sheet also accepts `sample:<member>:<fixture>` to inspect one
panel or feedback component at full viewport size, for example
`style:default;sample:tooltipCard:overflow`. Its all-fixtures `states` page
is a catalogue, not a substitute for an isolated state check.

The HUD sheet mounts `HudSheet`; its review command is
`style:default;page:<page>`, with the HUD pages listed below.

| Review area | Sheet and pages |
| --- | --- |
| HUD instruments (contained bars, centered rings/orbs, ordinary/warning/critical timers) | `sheets/hud`: `meters`, `objectives` |
| Abilities and touch/button states (cooldown, charges, locked/unaffordable, held/disabled, overflow) | `sheets/hud`: `actions`, `buttons`, `touch` |
| Compass and maps | `sheets/hud`: `navigation` |
| Radial commands (chosen, disabled, unaffordable) and map-layer legend (selected, unselected) | `sheets/hud`: `tactics` |
| Resource pips, boss/combo HUD, pickups, diagnostics and timed input | `sheets/hud-builtins`: `resources`, `resource-states`, `activity`, `activity-states`, `prompts`, `outcomes`, `waiting` |
| Panels and data (breadcrumbs, grid, tooltip comparison, menu, chart, graph, carousel) | `sheets/components`: `panels`, isolated `sample` fixtures |
| Feedback and loading (celebration, ribbon, skeletons, spinner, split panes, score thresholds, effects, action hints) | `sheets/components`: isolated `sample` fixtures |
| Cards and run maps | `sheets/card`: `table`, `builder`, `run` |
| Store, progression and social panels | `sheets/meta`: `store`, `progress`, `social`, `states` |
| Base states, selectable rows and progress steps | `sheets/components`: `sample:emptyState`, `sample:errorState`, `sample:listItem`, `sample:steps` with named fixtures (`base/README.md`) |
| Item slots and identity | `sheets/rpg`: `primitives` (`base/README.md`) |
| Pass rewards, reveals, inbox and news | `sheets/components`: `sample:passTrack`, `sample:reveal`, `sample:inbox`, `sample:news` with named fixtures (`meta/README.md`) |
| Tooltip comparison presence and long names | `sheets/components`: `sample:tooltipCard:comparison`, desktop side-by-side and phone stacked; signed deltas, added/removed stats |

Shooter and racing keep their own group READMEs and sheets. World
components keep anchor, ground and mass primitives; missing world captures
and their pending L4 review are recorded explicitly in the world sheet.

| Purpose | Documents |
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
| World tags over units and items | `world/nameplate`, `world/waypoint`, `world/loot-label`, `world/speech-bubble`, `world/interaction-prompt` |
| Ground marks (selection, telegraphs, paths, placement) | `world/selection-ring`, `world/telegraph`, `world/path-line`, `world/placement-ghost` |
| Mass world elements (overhead bars, damage numbers) | `world/overhead-bars`, `hud/damage-numbers` |
| RPG inventory, character and equipment (`rpg/README.md`) | `panels/backpack`, `panels/item-card`, `rpg/character`, `rpg/stats`, `rpg/menu-bar` |
| RPG quests, world map and conversation | `panels/quest-log`, `rpg/quest-detail`, `hud/quest-tracker`, `rpg/world-map`, `rpg/place-card`, `panels/conversation` |
| RPG party, turn order and combat HUD | `rpg/vitals`, `rpg/party`, `rpg/turn-order`, `hud/combat-log`, `rpg/enemy-info`, `rpg/commands` |
| Shooter HUD: aim, ammunition, damage direction, killfeed (`shooter/README.md`) | `hud/crosshair`, `hud/ammo-counter`, `hud/damage-direction`, `hud/killfeed`, `shooter/combat-hud` |
| Shooter loadout | `shooter/loadout` |
| Strategy selection, commands, production queue, waves, tower upgrades (`strategy/README.md`) | `strategy/selection`, `strategy/queue`, `strategy/wave`, `strategy/tower` |
| Racing telemetry, laps, standings, banner, garage, results (`racing/README.md`) | `racing/telemetry`, `racing/lap-hud`, `racing/standings`, `racing/banner`, `racing/garage`, `racing/results` |
| Card hand, piles, collection, deck, rewards, run map (`card/README.md`) | `card/face`, `card/hand`, `card/piles`, `card/collection`, `card/deck-list`, `card/reward`, `card/run-map` |
| Puzzle goals, boosters, level path, summary (`puzzle/README.md`) | `puzzle/goals`, `puzzle/boosters`, `puzzle/level-path`, `puzzle/summary` |
| Idle resources, generators, purchases, prestige, offline summary (`idle/README.md`) | `idle/resources`, `idle/generators`, `idle/purchases`, `idle/prestige`, `idle/offline` |
| Rhythm song and difficulty selection, score HUD, judgement, results (`rhythm/README.md`) | `rhythm/song-select`, `rhythm/difficulty`, `rhythm/score-hud`, `rhythm/judgement`, `rhythm/results` |
| Arena roster, draft, versus, lobby, round HUD, scoreboard (`arena/README.md`) | `arena/roster`, `arena/draft`, `arena/versus`, `arena/lobby`, `arena/round-hud`, `arena/scoreboard` |
| Arena ready checks | `arena/ready` |
| Arena map votes | `arena/vote` |
| Arena spectator controls | `arena/spectator` |
| Shop, achievements, daily quests and rewards, chat, friends (`meta/README.md`) | `meta/shop`, `meta/shop-confirm`, `meta/achievements`, `meta/daily-quests`, `meta/daily-rewards`, `meta/chat`, `meta/friends` |

A chip, a tag or a filter toggle has no component: it is a
`<button aria-pressed={on}>`. The attribute carries its state to a screen
reader and qualifies its part for a pack (`Shop::filter:checked`). The
default style dresses a button that is on with `--button-selected-fill`
and `--button-selected-ink` whatever its variant; a pack sets those tokens
or dresses `button[aria-pressed=true]::part(base)`.

`hud/touch-controls` is the whole on-screen pad: a `<joystick>` or `<dpad>` at
the start side, `hud/action-button`s at the end side and an optional second
stick, each an input source of the og-input action it names. It reports no
events and shows only where the primary pointer is a finger (`always`
shows it under a mouse too). Its parts are `frame`, `stick`, `actions` and
`look`.

## Parts

The `base/`, `hud/`, `world/` and `panels/` components' parts, as each declares them in its
`<meta name="parts">` (`AbilityBar.cell` in a selector). A part's role
name (`frame`, `band`, `title`, `row`, `cell`…) takes the default style's
role dressing (docs/ui.md §3.1). Each part's state qualifiers are listed after it: a class
(`.cooling`), an attribute (`[data-rarity]`, `[aria-selected]`) or
`:disabled`. A part that is a button also has the control's own hover,
pressed, focus-visible and disabled states (docs/ui.md §3.2).

| Component (document) | Parts |
| --- | --- |
| `AbilityBar` (`hud/ability-bar.svelte`) | `frame`, `cell` (a button: `.active` `.cooling` `.locked` `.tinted` `:disabled`), `well`, `icon`, `figure`, `badge` (`.short`), `label` |
| `AbilitySlot` (`hud/ability-slot.svelte`) | `frame` (`.ultimate`), `cell` (a button: `.active` `.cooling` `.full` `.locked` `.off` `.tinted` `:disabled`), `well`, `icon`, `ultimate`, `figure`, `percent`, `badge` (`.short`), `track`, `fill`, `label` |
| `ActionButton` (`hud/action-button.svelte`) | `disc` (a button: `.primary` `:disabled`), `charge`, `icon`, `text` |
| `ActionHints` (`hud/action-hints.svelte`) | `frame`, `row` (a button), `cap`, `label` |
| `Avatar` (`base/avatar.svelte`) | `frame`, `picture`, `clip`, `portrait`, `fallback`, `spinner`, `ring`, `level`, `name`, `status`, `mark`, `loading` |
| `Breadcrumbs` (`panels/breadcrumbs.svelte`) | `frame`, `crumb` (`.current`), `icon`, `link`, `current` (`[aria-current=page]`), `separator` |
| `Carousel` (`panels/carousel.svelte`) | `frame`, `cell`, `art`, `picture`, `well`, `title`, `body`, `prev` (a button: `:disabled` with fewer than two pages), `next` (a button: `:disabled` with fewer than two pages), `dot`, `lit` |
| `Celebration` (`hud/celebration.svelte`) | `ring`, `shell`, `stage`, `rays`, `halo`, `title`, `value` |
| `Chart` (`panels/chart.svelte`) | `frame`, `title`, `subtitle`, `tick`, `grid`, `plate`, `bar` (`.tinted`), `area`, `line`, `label` |
| `Compass` (`world/compass.svelte`) | `frame`, `window`, `poi` (`.ally` `.enemy` `.loot` `.objective` `.ping`), `distance`, `notch`, `badge`, `readout` |
| `Cooldown` (`hud/cooldown.svelte`) | `cell` (a button: `.ready` `:disabled`), `icon`, `figure` |
| `DamageNumbers` (`hud/damage-numbers.svelte`) | `figure` (`.tinted`) |
| `DataGrid` (`panels/data-grid.svelte`) | `frame`, `band`, `heading` (`.numeric`), `row` (a button: `[aria-selected]`), `value` (`.numeric`), `ghost`, `error`, `empty`, `mark`, `message`, `hint`, `retry` (a button) |
| `HeroButton` (`hud/hero-button.svelte`) | `base` (a button: `[variant]` primary, secondary or danger, `.danger` `.premium` `.secondary` `:disabled`), `label`, `key` |
| `HoldButton` (`hud/hold-button.svelte`) | `base` (a button: `.primary` `.secondary` `:disabled`), `icon`, `label`, `hint` |
| `HudTimer` (`hud/hud-timer.svelte`) | `frame` (`.critical` `.paused` `.warn`), `label`, `ring`, `figure` |
| `InteractionPrompt` (`world/interaction-prompt.svelte`) | `frame` (`.disabled` `:disabled`), `key`, `verb`, `name`, `reason` (`:disabled`) |
| `ItemSlot` (`base/item-slot.svelte`) | `frame`, `cell` (a button: `[aria-pressed]` `:disabled`), `icon`, `loading`, `count`, `cooldown`, `remaining`, `key`, `state`, `name` |
| `LootLabel` (`world/loot-label.svelte`) | `tag` (a button: `.epic` `.legendary` `.rare` `.uncommon`), `name`, `count` |
| `Menu` (`panels/menu.svelte`) | `frame`, `row` (a button: `.danger` `[aria-checked]` `:disabled`), `heading`, `separator`, `icon`, `check`, `dot`, `shortcut`, `chevron` |
| `MinimapView` (`world/minimap-view.svelte`) | `bezel`, `glass` (a button), `grid`, `marker` (`.ally` `.enemy` `.loot` `.objective` `.ping`), `player` (`.turning`), `north`, `badge` |
| `Minimap` (`world/minimap.svelte`) | `frame`, `marker` (`.accent` `.danger` `.info` `.player` `.success` `.warn`), `label`, `cell`, `grid` (only without a texture) |
| `Nameplate` (`world/nameplate.svelte`) | `frame` (`.boss` `.friendly` `.hostile` `.player` `.targeted`), `icon`, `badge`, `name`, `subtitle`, `well`, `fill`, `shield`, `cast`, `spell` |
| `NodeGraph` (`panels/node-graph.svelte`) | `well`, `link` (`.active`), `node` (a button: `.active` `.ally` `.current` `.diamond` `.done` `.enemy` `.hex` `.locked` `.selected` `.square` `[aria-disabled]` `:disabled`), `icon`, `label` |
| `OrbMeter` (`hud/orb-meter.svelte`) | `orb`, `fill`, `value`, `label` |
| `OverheadBars` (`world/overhead-bars.svelte`) | `bar` (`.tinted`) |
| `OverlayBar` (`hud/overlay-bar.svelte`) | `frame`, `title`, `ramp`, `label`, `bar`, `option` (a button: `[aria-pressed]`), `art`, `well`, `icon`, `name` |
| `PathLine` (`world/path-line.svelte`) | `line` (`.invalid`), `badge` |
| `PlacementGhost` (`world/placement-ghost.svelte`) | `reach`, `footprint`, `badge` |
| `ProgressBar` (`hud/progress-bar.svelte`) | `label`, `figure`, `track`, `buffer`, `fill` |
| `ProgressRing` (`hud/progress-ring.svelte`) | `ring`, `figure` |
| `RadialMenu` (`hud/radial-menu.svelte`) | `disc`, `slice` (`.chosen` `.disabled`), `item` / `cell` (one menuitem button: `[aria-current=true]` `:disabled`), `label`, `centre` / `well` (filled centre; empty `detail` shows the chosen label) |
| `RadialMeter` (`hud/radial-meter.svelte`) | `wheels` |
| `Ribbon` (`base/ribbon.svelte`) | `frame` (`.bottomLeft` `.bottomRight` `.center` `.danger` `.info` `.success` `.topLeft` `.warn`), `ribbon`, `text` |
| `SelectionRing` (`world/selection-ring.svelte`) | `ring` (`.hostile` `.hover` `.neutral` `.segmented`) |
| `SkeletonGroup` (`base/skeleton-group.svelte`) | `art`, `picture`, `line` |
| `SpeechBubble` (`world/speech-bubble.svelte`) | `frame`, `label`, `line`, `tail` |
| `Spinner` (`base/spinner.svelte`) | `ring` |
| `SplitView` (`panels/split-view.svelte`) | `frame`, `start`, `divider`, `grip`, `end` |
| `StarBar` (`hud/star-bar.svelte`) | `label`, `figure`, `track`, `well`, `fill`, `star` (`.lit`) |
| `StatBar` (`hud/stat-bar.svelte`) | `shield`, `frame` (`.bare` `.hud`), `label`, `track`, `fill`, `inside`, `figure` |
| `StatusGrid` (`hud/status-grid.svelte`) | `cell`, `badge`, `figure` |
| `Telegraph` (`world/telegraph.svelte`) | `area` (`.friendly`) |
| `TooltipCard` (`panels/tooltip-card.svelte`) | `frame` (`.common` `.epic` `.legendary` `.mythic` `.rare` `.uncommon`), `title`, `label`, `row` (`[data-change=added]` `[data-change=removed]`), `figure` (removed value muted), `well`, `badge`, `change` (translated Added/Removed), `delta`, `mark` (signed direction) |
| `TouchControls` (`hud/touch-controls.svelte`) | `frame` (`.always`), `stick` (`:disabled`), `actions`, `look` (`:disabled`) |
| `Waypoint` (`world/waypoint.svelte`) | `pin`, `label`, `figure` |

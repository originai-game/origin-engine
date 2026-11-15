# Component documents

Component documents built from og-ui's public elements and built into the
engine: the HUD, feedback, world and pattern pieces, and, in a directory
per group, a genre's components (`rpg/`, `shooter/`, `strategy/` and the rest, each with its
README). A game's document
uses any of them by its declared name (`<AbilityBar slots={$slots} />`,
`<RpgBackpack items={$items} />`) without a copy: the cook bundles it as
`@ui/components/<path>` and the screen that uses it loads it. Copy one with
`og ui eject <component>` (`og ui eject rpg/backpack` in a group) and edit
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

Expressions compute no geometry, so the game hands in what geometry a
component needs: a chart's line and a path line's route as SVG path data, a radial
menu's slice positions (`at`, `span`), a split view's ratio from its
divider's pointer events, a ground mark's world centre.

`og ui eject --list` prints these with the screens. The component document
test cooks every one from its fixture at desktop and phone sizes.

| Purpose | Documents |
|---|---|
| Ability bars, cooldowns and buffs | `ability-bar`, `ability-slot`, `cooldown`, `status-grid` |
| Meters and resource bars | `stat-bar`, `orb-meter`, `radial-meter` |
| Progress and loading placeholders | `progress-bar`, `progress-ring`, `skeleton-group`, `spinner` |
| Timers and scores | `hud-timer`, `star-bar` |
| Celebrations (confetti, a burst, a level up, fireworks) | `celebration` |
| Navigation (minimap, compass) | `minimap`, `minimap-view`, `compass` |
| Menus and layout (radial menu, split view, carousel, breadcrumbs) | `radial-menu`, `split-view`, `carousel`, `breadcrumbs` |
| Menus and tooltip cards (in a `popover` or `contextmenu` popover) | `menu`, `tooltip-card` |
| Charts and map overlays | `chart`, `overlay-bar` |
| Skill trees and run maps | `node-graph` |
| Data views (inventories, level selects) | `data-grid` |
| Buttons and badges (hero button, touch action, hold to confirm, action bar, ribbon) | `hero-button`, `action-button`, `hold-button`, `action-hints`, `ribbon` |
| Touch controls (a stick or d-pad and the action buttons, shown under a finger) | `touch-controls` |
| World tags over units and items | `nameplate`, `waypoint`, `loot-label`, `speech-bubble`, `interaction-prompt` |
| Ground marks (selection, telegraphs, paths, placement) | `selection-ring`, `telegraph`, `path-line`, `placement-ghost` |
| Mass world elements (overhead bars, damage numbers) | `overhead-bars`, `damage-numbers` |
| RPG inventory, character and equipment (`rpg/README.md`) | `rpg/backpack`, `rpg/item-card`, `rpg/character`, `rpg/stats`, `rpg/menu-bar` |
| RPG quests, world map and conversation | `rpg/quest-log`, `rpg/quest-detail`, `rpg/quest-tracker`, `rpg/world-map`, `rpg/place-card`, `rpg/conversation`, `rpg/compass` |
| RPG party, turn order and combat HUD | `rpg/vitals`, `rpg/party`, `rpg/turn-order`, `rpg/action-bar`, `rpg/combat-log`, `rpg/enemy-info`, `rpg/commands` |
| Shooter HUD: aim, ammunition, damage direction, killfeed (`shooter/README.md`) | `shooter/crosshair`, `shooter/ammo-counter`, `shooter/damage-direction`, `shooter/killfeed`, `shooter/combat-hud` |
| Shooter loadout | `shooter/loadout` |
| Strategy selection, commands, production queue, waves, tower upgrades (`strategy/README.md`) | `strategy/selection`, `strategy/queue`, `strategy/wave`, `strategy/tower` |
| Racing telemetry, laps, standings, banner, garage, results (`racing/README.md`) | `racing/telemetry`, `racing/lap-hud`, `racing/standings`, `racing/banner`, `racing/garage`, `racing/results` |
| Card hand, piles, collection, deck, rewards, run map (`card/README.md`) | `card/face`, `card/hand`, `card/piles`, `card/collection`, `card/deck-list`, `card/reward`, `card/run-map` |
| Puzzle goals, boosters, level path, summary (`puzzle/README.md`) | `puzzle/goals`, `puzzle/boosters`, `puzzle/level-path`, `puzzle/summary` |
| Idle resources, generators, purchases, prestige, offline summary (`idle/README.md`) | `idle/resources`, `idle/generators`, `idle/purchases`, `idle/prestige`, `idle/offline` |
| Rhythm song and difficulty selection, score HUD, judgement, results (`rhythm/README.md`) | `rhythm/song-select`, `rhythm/difficulty`, `rhythm/score-hud`, `rhythm/judgement`, `rhythm/results` |
| Arena roster, draft, versus, lobby, round HUD, scoreboard (`arena/README.md`) | `arena/roster`, `arena/draft`, `arena/versus`, `arena/lobby`, `arena/round-hud`, `arena/scoreboard` |
| Shop, achievements, daily quests and rewards, chat, friends (`general/README.md`) | `general/shop`, `general/shop-confirm`, `general/achievements`, `general/daily-quests`, `general/daily-rewards`, `general/chat`, `general/friends` |

A chip, a tag or a filter toggle has no component: it is a
`<button aria-pressed={on}>`. The attribute carries its state to a screen
reader and qualifies its part for a pack (`Shop::filter:checked`). The
default style dresses a button that is on with `--button-selected-fill`
and `--button-selected-ink` whatever its variant; a pack sets those tokens
or dresses `button[aria-pressed=true]::part(base)`.

`touch-controls` is the whole on-screen pad: a `<joystick>` or `<dpad>` at
the start side, `action-button`s at the end side and an optional second
stick, each an input source of the og-input action it names. It reports no
events and shows only where the primary pointer is a finger (`always`
shows it under a mouse too). Its parts are `frame`, `stick`, `actions` and
`look`.

## Parts

The ungrouped components' parts, as each declares them in its
`<meta name="parts">` (`AbilityBar.cell` in a selector). A part's role
name (`frame`, `band`, `title`, `row`, `cell`…) takes the default style's
role dressing (docs/ui.md §3.1). Each part's state qualifiers are listed after it: a class
(`.cooling`), an attribute (`[data-rarity]`, `[aria-selected]`) or
`:disabled`. A part that is a button also has the control's own hover,
pressed, focus-visible and disabled states (docs/ui.md §3.2).

| Component (document) | Parts |
| --- | --- |
| `AbilityBar` (`ability-bar.svelte`) | `frame`, `cell` (a button: `.active` `.cooling` `.locked` `.tinted` `:disabled`), `well`, `icon`, `figure`, `badge` (`.short`), `label` |
| `AbilitySlot` (`ability-slot.svelte`) | `frame` (`.ultimate`), `cell` (a button: `.active` `.cooling` `.full` `.locked` `.off` `.tinted` `:disabled`), `well`, `icon`, `ultimate`, `figure`, `percent`, `badge` (`.short`), `track`, `fill`, `label` |
| `ActionButton` (`action-button.svelte`) | `disc` (a button: `.primary` `:disabled`), `charge`, `icon`, `text` |
| `ActionHints` (`action-hints.svelte`) | `frame`, `row` (a button), `cap`, `label` |
| `Breadcrumbs` (`breadcrumbs.svelte`) | `frame`, `crumb` (`.current`), `icon`, `link` (`[aria-current]`), `current`, `separator` |
| `Carousel` (`carousel.svelte`) | `frame`, `cell`, `art`, `well`, `title`, `body`, `prev` (a button), `next` (a button), `dot`, `lit` |
| `Celebration` (`celebration.svelte`) | `ring`, `shell`, `stage`, `rays`, `halo`, `title`, `value` |
| `Chart` (`chart.svelte`) | `frame`, `title`, `subtitle`, `tick`, `grid`, `bar` (`.tinted`), `area`, `line`, `label` |
| `Compass` (`compass.svelte`) | `frame`, `window`, `poi` (`.ally` `.enemy` `.loot` `.objective` `.ping`), `distance`, `notch`, `badge` |
| `Cooldown` (`cooldown.svelte`) | `cell` (a button: `.ready` `:disabled`), `icon`, `figure` |
| `DamageNumbers` (`damage-numbers.svelte`) | `figure` (`.tinted`) |
| `DataGrid` (`data-grid.svelte`) | `frame`, `band`, `heading` (`.numeric`), `row` (a button: `[aria-selected]`), `value` (`.numeric`), `ghost`, `error`, `empty`, `mark`, `message`, `hint`, `retry` (a button) |
| `HeroButton` (`hero-button.svelte`) | `base` (a button: `.danger` `.premium` `.secondary` `:disabled`), `label`, `key` |
| `HoldButton` (`hold-button.svelte`) | `base` (a button: `.primary` `.secondary` `:disabled`), `icon`, `label`, `hint` |
| `HudTimer` (`hud-timer.svelte`) | `frame` (`.critical` `.paused` `.warn`), `label`, `ring`, `figure` |
| `InteractionPrompt` (`interaction-prompt.svelte`) | `frame` (`.disabled` `:disabled`), `key`, `verb`, `name`, `reason` (`:disabled`) |
| `LootLabel` (`loot-label.svelte`) | `tag` (a button: `.epic` `.legendary` `.rare` `.uncommon`), `name`, `count` |
| `Menu` (`menu.svelte`) | `frame`, `row` (a button: `.danger` `[aria-checked]` `:disabled`), `heading`, `separator`, `icon`, `check`, `dot`, `shortcut`, `chevron` |
| `MinimapView` (`minimap-view.svelte`) | `bezel`, `glass` (a button), `grid`, `marker` (`.ally` `.enemy` `.loot` `.objective` `.ping`), `player` (`.turning`), `north`, `badge` |
| `Minimap` (`minimap.svelte`) | `frame`, `marker` (`.accent` `.danger` `.info` `.player` `.success` `.warn`), `label` |
| `Nameplate` (`nameplate.svelte`) | `frame` (`.boss` `.friendly` `.hostile` `.player` `.targeted`), `icon`, `badge`, `name`, `subtitle`, `well`, `fill`, `shield`, `cast`, `spell` |
| `NodeGraph` (`node-graph.svelte`) | `well`, `link` (`.active`), `node` (a button: `.active` `.ally` `.current` `.diamond` `.done` `.enemy` `.hex` `.locked` `.selected` `.square` `[aria-disabled]` `:disabled`), `icon`, `label` |
| `OrbMeter` (`orb-meter.svelte`) | `orb`, `fill`, `value`, `label` |
| `OverheadBars` (`overhead-bars.svelte`) | `bar` (`.tinted`) |
| `OverlayBar` (`overlay-bar.svelte`) | `frame`, `title`, `ramp`, `label`, `bar`, `option` (a button: `[aria-pressed]`), `art`, `well`, `icon`, `name` |
| `PathLine` (`path-line.svelte`) | `line` (`.invalid`), `badge` |
| `PlacementGhost` (`placement-ghost.svelte`) | `reach`, `footprint`, `badge` |
| `ProgressBar` (`progress-bar.svelte`) | `label`, `figure`, `track`, `buffer`, `fill` |
| `ProgressRing` (`progress-ring.svelte`) | `ring`, `figure` |
| `RadialMenu` (`radial-menu.svelte`) | `disc`, `slice` (`.chosen` `.disabled` `:disabled`), `label`, `centre` |
| `RadialMeter` (`radial-meter.svelte`) | `wheels` |
| `Ribbon` (`ribbon.svelte`) | `frame` (`.bottomLeft` `.bottomRight` `.center` `.danger` `.info` `.success` `.topLeft` `.warn`), `ribbon`, `text` |
| `SelectionRing` (`selection-ring.svelte`) | `ring` (`.hostile` `.hover` `.neutral` `.segmented`) |
| `SkeletonGroup` (`skeleton-group.svelte`) | `art`, `line` |
| `SpeechBubble` (`speech-bubble.svelte`) | `frame`, `label`, `line`, `tail` |
| `Spinner` (`spinner.svelte`) | `ring` |
| `SplitView` (`split-view.svelte`) | `frame`, `start`, `divider`, `grip`, `end` |
| `StarBar` (`star-bar.svelte`) | `label`, `figure`, `track`, `fill`, `star` (`.lit`) |
| `StatBar` (`stat-bar.svelte`) | `shield`, `frame` (`.bare` `.hud`), `label`, `track`, `fill`, `inside`, `figure` |
| `StatusGrid` (`status-grid.svelte`) | `cell`, `badge`, `figure` |
| `Telegraph` (`telegraph.svelte`) | `area` (`.friendly`) |
| `TooltipCard` (`tooltip-card.svelte`) | `frame` (`.common` `.epic` `.legendary` `.mythic` `.rare` `.uncommon`), `title`, `label`, `row`, `figure`, `well`, `badge` |
| `TouchControls` (`touch-controls.svelte`) | `frame` (`.always`), `stick` (`:disabled`), `actions`, `look` (`:disabled`) |
| `Waypoint` (`waypoint.svelte`) | `pin`, `label`, `figure` |

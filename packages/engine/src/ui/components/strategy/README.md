# strategy: built-in components

The RTS and tower-defence HUD (the resource bar, the match clock, the wave,
alerts, control groups, the minimap, the selection panel, the production
queue, the command card) and the tower-upgrade inspector, as documents over
core controls (docs/ui.md §4). They draw what the game tells them and return
intents with stable row keys. They never compute costs, build times,
cooldowns, legality, income, wave timing, rewards, refunds or stats: every
figure arrives formatted from the game, and every flag (enabled, affordable,
full, active, alert, can call early, can sell) is the game's decision.

They are built into the engine: a game's page uses one by name
(`<StrategyCommands commands={$commands} ... />`) and loading the page loads
it. `og ui eject strategy/commands` copies one into `assets/ui/strategy/` to
change it; the game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds layout only. The HUD members stand bare over
the scene in the default style's smoked glass and lit edges: every colour,
fill, rim, radius and type step is a `--hud-*` or `--bar-*` role token (a
strategy set beside the shooter's, the RPG's and the racing HUD's: a plate
lit in danger, an empty slot, a unit's health green, gold and red, the
minimap's camera frame, and the readout, unit-name and cooldown type
steps), and every type step is in font steps, so it follows the player's
text scale. Pictures, portraits, cards and command tiles are art: they keep
their size at any text scale. The HUD's accent (`--hud-accent`) lights the
focused card, the order in hand, the group selected, the info card's edge
and your own dots on the minimap; a game sets it, with the tokens made from
it, to its key colour. Unit, command and order names stay on one line and
scroll when they overflow. `StrategyTower` keeps its panel layout, dressed
in the same glass.

Hooks for a pack: `data-warn` on a resource and `data-trend` (gain, loss,
none, note) on its rate; `data-paused` on the clock; `data-phase`
(countdown, active, done), `data-final` and `data-near` on the wave frame,
`data-boss` and `data-scouted` on its enemy rows; `data-kind` (danger, done,
info) and `data-faded` on alerts; `aria-pressed` and `data-alert` on control
groups; `data-kind` (own, ally, foe, neutral) and `data-base` on minimap
dots; `data-kind` on the selection frame and cards, `aria-selected` on the
focused card, `data-tone` (ok, hurt, low) on health bars; `data-state`
(building, waiting, paused, blocked) on queue tiles and `data-full` on the
frame and its count; `data-active`, `data-cooling`, `data-affordable`,
`data-enabled` on command tiles (and `:disabled`); `data-state`
(affordable, unaffordable, locked, maxed) on tower paths, their buy button,
cost and badge, and `data-next` on stat rows. A state is always also a word
or a figure (Near cap, Paused, Under attack, All waves cleared, Final wave,
Boss, Not yet scouted, Queue full, Blocked and its reason, the price on a
red tag, the cooldown's seconds, Too costly, Locked, Maxed), never colour
alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `StrategyResources` (`resources.svelte`) | `resources` keyed `{ id, name, icon, glyph, amount, cap, rate, trend, note, warn }`, `tier` | none | `frame`, `resource`, `picture`, `amount`, `cap`, `rate`, `tier` |
| `StrategyClock` (`clock.svelte`) | `label`, `time`, `speed`, `paused` | none | `frame`, `label`, `time`, `speed` |
| `StrategyWave` (`wave.svelte`) | `wave`, `waves` (0: endless), `name`, `icon`, `phase` (countdown, active, done), `next` (seconds; negative hides), `nextTotal`, `warnAt` (optional, 5), `finalWave`, `enemies` keyed `{ id, name, icon, count, boss, scouted }`, `canCallEarly`, `reward` | `callEarly` | `frame`, `plate`, `icon`, `label`, `time`, `bar`, `fill`, `state`, `badge`, `row`, `mark`, `name`, `count`, `call`, `reward` |
| `StrategyAlerts` (`alerts.svelte`) | `alerts` keyed `{ id, kind (danger, done, info), text, key, faded }`, newest first | none | `frame`, `alert`, `mark`, `text`, `key` |
| `StrategyGroups` (`groups.svelte`) | `groups` keyed `{ id, label, key, icon, count, active, alert }` | `pick` (the group's key) | `frame`, `group`, `key`, `picture`, `count` |
| `StrategyMinimap` (`minimap.svelte`) | `map` (texture), `dots` keyed `{ id, kind, x, y, base, label }`, `viewX`, `viewY`, `viewWidth` (0 hides), `viewHeight`, `pings` keyed `{ id, x, y, label }`; positions 0 to 1 | none | `frame`, `map`, `dot`, `view`, `ping` |
| `StrategySelection` (`selection.svelte`) | `name` (empty: Nothing selected), `count`, `kind`, `line`, `status`, `portrait`, `icon`, `health`, `healthMax` (0 hides), `stats` keyed `{ id, label, value }`, `units` keyed `{ id, name, kind, portrait, icon, count, health, healthMax }` (cards for two or more), `focused` | `choose` (a card's key) | `frame`, `portrait`, `health`, `fill`, `vital`, `name`, `count`, `line`, `status`, `stat`, `roster`, `cell`, `art`, `meter`, `badge`, `more` (a phone's +n tile) |
| `StrategyQueue` (`queue.svelte`) | `title` (empty: Production), `items` keyed `{ id, name, picture, icon, count, progress 0–1, timeText, state (building, waiting, paused, blocked), reason }` first in production first, `queued`, `limit` (0 hides the count and free places), `full` | `cancel` (a tile's key) | `frame`, `title`, `slot`, `art`, `sweep`, `time`, `badge`, `open`, `count`, `state`, `reason` |
| `StrategyCommands` (`commands.svelte`) | `commands` keyed `{ id, name (empty: an empty cell), icon, glyph, key, action, enabled, affordable, cost, costIcon, cooldown 0–1, time, active, detail, cooldownText, readyText, reason }` in grid order, `focused` (the command whose info card shows) | `command` (a tile's key; a disabled or unaffordable tile sends nothing) | `frame`, `info`, `title`, `detail`, `price`, `tag`, `grid`, `command`, `art`, `key`, `sweep`, `time`, `cost`, `blank` |
| `StrategyTower` (`tower.svelte`) | `name` (empty: Tower), `icon`, `portrait`, `level` (0 hides), `stats` keyed `{ id, label, value, next }`, `paths` keyed `{ id, name, icon, detail, tier, tiers, cost, state (affordable, unaffordable, locked, maxed), reason }`, `sellText`, `canSell` | `upgrade` (the path's key; only an affordable path is enabled), `sell` | `frame`, `band`, `title`, `badge`, `portrait`, `row`, `path`, `name`, `cost`, `reason` |

The command card shows each tile's key cap from its `key`, or from its
`action` (the bound key's glyph on the device used last,
`og.input.defineAction`); the caps hide on a coarse pointer, and the game
binds and dispatches the hotkeys itself. A cooldown is a dark sweep turning
clockwise with `time` on it; the game decides whether a cooling command is
also disabled. The HUD members lay out by the screen: on a phone (narrower
than 600 px) the resource bar takes the width on one or two rows, the
minimap is smaller, the selection drops its stats and its cards scroll in
one row, the command card takes the width eight tiles to a row without the
info card, and the queue's tiles are smaller. The tower lays out by its own
width (container queries on the frame) and keeps its paths inside the
height its parent gives it; Sell remains outside that scroller.

Every member carries a `fixture:default` and the states that apply
(resources: tier, capped, empty, overflow; clock: paused, empty, overflow;
wave: incoming, active, final, done, empty, overflow; alerts: info, empty,
overflow; groups: idle, empty, overflow; minimap: allies, empty, overflow;
selection: single, empty, groups, dense, overflow; queue: blocked, empty,
full, paused, overflow; commands: unaffordable, cooling, empty, overflow;
tower: maxed, disabled, empty, overflow). Loading and error do not apply:
the members draw data the game already has.

## Review sheet and tests

`og ui sheet strategy` renders `packages/engine/sheets/strategy/`, dressed
by the fantasy pack: `field` (the normal page: the RTS HUD as its approved
mockup over a battlefield, with sample art), `defence` (the stress page: an
endless wave with five-digit counts, nine-digit stockpiles, an overflowing
queue and a tier-ten tower with long names) and `states` (every member in
every fixture state). Each page also has a `-default` screen with its
explicit `style:default;page:<page>` commands.

`bun test tests/strategy.test.ts` (from `packages/engine`) mounts every
member by name from a game page in every fixture at desktop and phone, in
two packs and in the default style at 100 % and 150 % text, each inside the
screen's width; it checks the declared parts, the intents and their keys,
the disabled, unaffordable and refused states, keyed selection through a
reorder, keyboard focus and confirm, and that each state is a word.

# strategy: built-in components

Selection and commands, the production queue, waves and the tower-upgrade
inspector, as documents over core controls (docs/ui.md §4). They draw what
the game tells them and return intents with stable row keys. They never
compute costs, build times, cooldowns, legality, wave timing, rewards,
refunds or stats: every figure arrives formatted from the game, and every
flag (enabled, full, affordable, locked, maxed, can call early, can sell)
is the game's decision.

They are built into the engine: a game's page uses one by name
(`<StrategyQueue items={$queue} ... />`) and loading the page loads it, with
the `StatBar`, `ProgressBar` and `HudTimer` it uses. `og ui eject
strategy/selection` copies one into `assets/ui/strategy/` to change it; the
game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing from
tokens. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`) supplies every appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `StrategyTower::path[data-state=locked]` or
`button[data-active=true]::part(StrategySelection\.command)`. The fantasy
pack's dress is `templates/styles/fantasy/strategy.css`.

Hooks for a pack: `data-kind` (unit, building) on selection cells and the
detail; `aria-selected` on the focused cell; `data-active` and
`data-cooling` on command plates (and `:disabled`); `data-full` on the
queue frame and its count badge, `data-state` (building, waiting, paused,
blocked) on queue rows and their badge; `data-phase` (countdown, active,
done) and `data-final` on the wave frame and the phase badge, `data-boss`
and `data-scouted` on enemy rows; `data-state` (affordable, unaffordable,
locked, maxed) on tower paths, their buy button, cost and badge, and
`data-next` on stat rows that show an upgraded value. A state is always also
a word (On, Waiting, Paused, Blocked, Queue full, Get ready, Under attack,
All waves cleared, Final wave, Boss, Not yet scouted, Too costly, Locked,
Maxed) or the game's reason, never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `StrategySelection` (`selection.svelte`) | `title` (empty: Selection), `summary`, `units` keyed `{ id, name, kind (unit, building), line, status, portrait (texture, 0 none), icon, count, health, healthMax }`, `focused` (the active subgroup's key), `commands` keyed `{ id, label, icon, action, hotkey, enabled, reason, cooldown 0–1, cooldownText, active }` | `choose` (a cell's key), `command` (a plate's key; a disabled plate sends nothing) | `frame`, `band`, `title`, `badge`, `cell`, `portrait`, `meter`, `detail`, `name`, `grid`, `command`, `key`, `reason` |
| `StrategyQueue` (`queue.svelte`) | `title` (empty: Production), `items` keyed `{ id, name, icon, count, progress 0–1, timeText, state (building, waiting, paused, blocked), reason }` first in production first, `queued`, `limit` (0 hides the count), `full` | `cancel` (the order's key) | `frame`, `band`, `title`, `badge`, `row`, `cell`, `name`, `reason` |
| `StrategyWave` (`wave.svelte`) | `wave`, `waves` (0: endless), `phase` (countdown, active, done), `next` (seconds; negative hides the `HudTimer`), `nextTotal`, `warnAt` (seconds left when the timer turns warn; optional, 5), `finalWave`, `enemies` keyed `{ id, name, icon, count, boss, scouted }`, `canCallEarly`, `reward` | `callEarly` (hidden when the game refuses it) | `frame`, `cell`, `title`, `badge`, `row`, `name` |
| `StrategyTower` (`tower.svelte`) | `name` (empty: Tower), `icon`, `portrait`, `level` (0 hides), `stats` keyed `{ id, label, value, next }` (`next` shows after an arrow, → 51), `paths` keyed `{ id, name, icon, detail, tier, tiers, cost, state (affordable, unaffordable, locked, maxed), reason }`, `sellText`, `canSell` | `upgrade` (the path's key; only an affordable path is enabled), `sell` (disabled when `canSell` is false) | `frame`, `band`, `title`, `badge`, `portrait`, `row`, `path`, `name`, `cost`, `reason` |

The command card shows each plate's key cap from its `action` (the bound
key's glyph on the device used last, `og.input.defineAction`) or its
`hotkey` label; the caps hide on a coarse pointer. The game binds the
hotkeys and dispatches them itself; a tap or a confirm on a plate reports
`command`. A cooldown shades the plate from the foot by `cooldown` and
shows `cooldownText`; the game decides whether a cooling command is also
disabled. The selection and the tower lay out by their own width (container
queries on the frame): in a narrow panel the selection stacks the roster and
detail over a full-width command card and Sell takes the width. The wave's
Call early takes the width by a media query: its frame is sized by its
contents.

Every member carries a `fixture:default` and the states that apply
(selection: single, empty, disabled, overflow; queue: empty, full, paused,
overflow; wave: active, final, done, empty, overflow; tower: maxed,
disabled, empty, overflow). Loading and error do not apply: the members
draw data the game already has.

## Review sheet and tests

`og ui sheet strategy` renders `packages/engine/sheets/strategy/`, dressed
by the fantasy pack: `field` (the normal page: the queue, the wave readout
and the selection with its command card over a bright meadow and a dark
forest), `defence` (the stress page: an endless wave with five-digit
counts, an overflowing queue and a tier-ten tower with long names) and
`states` (every member in every fixture state). `style:default` is a sheet
command for the default style.

`bun test tests/strategy.test.ts` (from `packages/engine`) mounts every
member by name from a game page in every fixture at desktop and phone,
checks the declared parts, the intents and their keys, the disabled and
refused states, keyed selection through a reorder, keyboard focus and
confirm, and that each state is a word.

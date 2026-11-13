# idle: built-in components

The resource bar, generators, the upgrade shop, prestige and the
welcome-back panel of an idle or incremental game, as documents over core
controls (docs/ui.md §4). They draw what the game tells them and return
intents with stable row keys. They never compute income, cycles, prices,
bulk-buy counts, prestige gains or offline earnings: every figure arrives
from the game, formatted or as a number the component only formats
(`formatCompact`, `formatDuration`).

They are built into the engine: a game's page uses one by name
(`<IdleResources currencies={$currencies} loading={false} />`) and loading
the page loads it, with the `ProgressBar` it uses. `og ui eject
idle/generators` copies one into `assets/ui/idle/` to change it; the game's
copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing from
tokens. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`) supplies every appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `IdleResources::row[data-full=true]` or
`button::part(IdleGenerators\.row)[data-locked=true]`. The fantasy pack's
dress is `templates/styles/fantasy/idle.css`.

Hooks for a pack: `data-full`, `data-loading` and `data-trend` (up, down,
flat) on resource rows, `data-trend` on their rate; `aria-selected`,
`data-locked`, `data-auto` and `data-boosted` on generator rows and
`data-tag` (level, boost, auto, locked) on their badges; `aria-checked` and
`data-amount` on the purchase amounts, `data-affordable`, `data-maxed` and
`data-locked` on purchase rows and `data-tag` (level, maxed, locked, short)
on their badges; `data-met` and `data-confirming` on the prestige frame,
`data-met` on its requirement and badge, `data-tag` (held, gain, now, after)
on its figures and cells. A state is always also a word (Full, Auto, Locked,
Not enough, Maxed, Ready, Not yet), never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `IdleResources` (`resources.svelte`) | `currencies` keyed `{ id, icon, name, amount, value, rate, perSecond, full }` (`value` and `rate` are the game's strings and win; empty shows `amount` and `perSecond` through `formatCompact`; a 0 income hides the rate; `perSecond` also sets `data-trend` whether or not `rate` is given), `loading` (placeholders for the figures; each row's name says it is loading) | none | `frame`, `row`, `name`, `figure`, `rate`, `badge`, `placeholder` |
| `IdleGenerators` (`generators.svelte`) | `title` (empty: Generators), `generators` keyed `{ id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }` (`cycle` 0–1, negative hides the `ProgressBar`), `selected` | `choose` (row key; a locked row is disabled and silent) | `frame`, `band`, `title`, `row`, `name`, `figure`, `rate`, `meter`, `badge` |
| `IdlePurchases` (`purchases.svelte`) | `title` (empty: Upgrades), `amounts` keyed `{ id, label }` (`label` is the game's wording and wins; empty reads the key `max` as Max and any other key as ×key; an empty list hides the choice), `amount`, `upgrades` keyed `{ id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }` | `chooseamount` (the amount's key, a radio group), `buy` (row key; only when affordable and neither maxed nor locked) | `frame`, `band`, `title`, `amount`, `row`, `name`, `effect`, `cost`, `badge` |
| `IdlePrestige` (`prestige.svelte`) | `title` (empty: Prestige), `currency`, `icon`, `held`, `gain`, `rows` keyed `{ id, icon, label, now, after }`, `requirement`, `progress` (negative hides the bar), `progressText`, `met`, `confirming` | `request` (Prestige; disabled until `met`), then while the game sets `confirming`: `confirm` (Reset) and `cancel` (Cancel, Escape, pad B) | `frame`, `band`, `title`, `figure`, `row`, `name`, `cell`, `requirement`, `badge`, `confirm` |
| `IdleOffline` (`offline.svelte`) | `title` (empty: Welcome back), `away` (the game's words; empty shows `seconds` through `formatDuration`), `seconds`, `earned` keyed `{ id, icon, name, amount }`, `canDouble` (false hides Double), `doubleText` | `collect`, `double` | `frame`, `band`, `title`, `summary`, `row`, `name`, `figure` |

Prestige asks in place rather than in a modal: `request` tells the game,
the game sets `confirming`, and Reset and Cancel replace the Prestige
button, Cancel first so no key resets by accident; Escape and pad B cancel
only while it asks. Narrow layouts come from the members' own queries:
IdlePurchases, IdleGenerators and IdlePrestige ask their own width
(container queries, so a side column on a wide screen narrows them too):
purchases put the amount choice under the title and each row's price and
Buy under its name; the generator and prestige rows wrap their figures
under the name and the actions share the width. IdleResources and
IdleOffline tighten on a phone's viewport.
The loading placeholder is a still bar in the documents; a pack may animate
it through `IdleResources.placeholder` (the fantasy pack breathes it on
`--motion-duration-slow` and holds it still under reduced motion).

Every member carries a `fixture:default` and the states that apply
(resources: loading, full, draining, empty, overflow; generators: empty,
disabled, selected, overflow; purchases: max, disabled, maxed, empty,
overflow; prestige: disabled, confirming, empty, overflow; offline:
disabled, empty, overflow).

## Review sheet and tests

`og ui sheet idle` renders the three review pages from
`packages/engine/sheets/idle/`, dressed by the fantasy pack: play (the
resource bar over generators and the upgrade shop), reset (prestige asking
to confirm and the welcome-back panel, then their long forms) and states
(every member in every fixture state, a scrolling page). The commands
`page:states-resources`, `-generators`, `-purchases`, `-prestige` and
`-offline` show one member's states on one desktop screen for `og ui
preview --screen "x=style:fantasy;page:states-purchases"`.
`src/fixtures.ts` repeats the documents' fixtures for the sheet's stores.

`bun test tests/idle.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, checks that
nothing spills past the screen, the declared parts, the intents and their
row keys, the disabled and refused states, the formatted-value-wins rule,
an amount's own label winning over its key, the prestige confirm flow with
Escape, and keyboard focus to Collect.

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
the page loads it. `og ui eject
idle/generators` copies one into `assets/ui/idle/` to change it; the game's
copy of the same name wins.

## Structure and appearance

The documents use HTML elements with CSS: `:host` styles the root,
`{#if}` mounts optional content, and parts and classes select the inner
boxes. Scrolling is `overflow-y: auto` on a `div`. Element ids and parts
remain stable when a game ejects a document.

Each member's `<style>` supplies layout and a token-based default look:
type hierarchy, spacing and quiet row surfaces, in the `og-components`
layer. A style pack (`og-pack`) overrides appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `IdleResources::row[data-full=true]` or
`button::part(IdleGenerators\.row)[data-locked=true]`. The fantasy pack dresses them
through its tokens alone (`templates/styles/fantasy/base.css`).

Hooks for a pack: `data-full`, `data-loading` and `data-trend` (up, down,
flat) on resource rows, `data-trend` on their rate; `aria-selected`,
`data-locked`, `data-auto` and `data-boosted` on generator rows and
`data-tag` (boost, auto, locked) on their tags; `aria-checked` and
`data-amount` on the purchase amounts, `data-affordable`, `data-maxed` and
`data-locked` on purchase rows and `data-tag` (level, maxed, locked, short)
on their badges; `data-met` and `data-confirming` on the prestige frame,
`data-met` on its requirement, tag and fill, `data-tag` (held, gain, now,
after) on its figures and readouts. A state is always also a word (Full, Auto, Locked,
Not enough, Maxed, Ready, Not yet), never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `IdleResources` (`resources.svelte`) | `currencies` keyed `{ id, icon, name, amount, value, rate, perSecond, full }` (`value` and `rate` are the game's strings and win; empty shows `amount` and `perSecond` through `formatCompact`, the amount counting to each new value; a 0 income hides the rate; `perSecond` also sets `data-trend` whether or not `rate` is given), `loading` (placeholders for the figures; each row's name says it is loading) | none | `frame`, `row`, `socket`, `name`, `figure`, `rate`, `tag`, `placeholder` |
| `IdleGenerators` (`generators.svelte`) | `title` (empty: Generators), `generators` keyed `{ id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }` (`cycle` 0–1, negative hides the cycle bar; on a locked row `rate` is what unlocks it), `selected` | `choose` (row key; a locked row is disabled and silent) | `frame`, `band`, `kicker`, `title`, `row`, `socket`, `level`, `name`, `owned`, `rate`, `track`, `fill`, `time`, `tag`, `fade` |
| `IdlePurchases` (`purchases.svelte`) | `title` (empty: Upgrades), `amounts` keyed `{ id, label }` (`label` is the game's wording and wins; empty reads the key `max` as Max and any other key as ×key; an empty list hides the choice), `amount`, `upgrades` keyed `{ id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }` | `chooseamount` (the amount's key, a radio group), `buy` (row key; only when affordable and neither maxed nor locked) | `frame`, `band`, `title`, `amount`, `row`, `socket`, `name`, `effect`, `cost`, `badge`, `figure`, `caption`, `fade` |
| `IdlePrestige` (`prestige.svelte`) | `title` (empty: Prestige), `currency`, `icon`, `held`, `gain`, `rows` keyed `{ id, icon, label, now, after }`, `requirement`, `progress` (negative hides the bar), `progressText`, `met`, `confirming` | `request` (Prestige; disabled until `met`), then while the game sets `confirming`: `confirm` (Reset) and `cancel` (Cancel, Escape, pad B) | `frame`, `band`, `kicker`, `title`, `socket`, `label`, `figure`, `row`, `name`, `readout`, `requirement`, `tag`, `track`, `fill`, `confirm` |
| `IdleOffline` (`offline.svelte`) | `title` (empty: Welcome back), `away` (the game's words; empty shows `seconds` through `formatDuration`), `seconds`, `earned` keyed `{ id, icon, name, amount }`, `canDouble` (false hides Double), `doubleText` | `collect`, `double` | `frame`, `band`, `kicker`, `title`, `away`, `figure`, `label`, `haul`, `socket`, `amount`, `name` |

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

## Gallery and tests

`bun packages/engine/src/node/gallery.ts --only idle/` renders every member's fixtures under every pack (docs/ui.md §6.5).

`bun test tests/idle.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, checks that
nothing spills past the screen, the declared parts, the intents and their
row keys, the disabled and refused states, the formatted-value-wins rule,
an amount's own label winning over its key, the prestige confirm flow with
Escape, and keyboard focus to Collect.

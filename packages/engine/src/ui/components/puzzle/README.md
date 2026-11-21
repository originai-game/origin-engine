# puzzle: built-in components

The level path, the in-level goals and moves, the booster tray and the level
summary of a level-based puzzle game (match-three, sliding, physics), as
documents over core controls (docs/ui.md §4). They draw what the game tells
them and return intents with stable row keys. They never compute scores,
stars, moves, which levels are open, whether a booster applies, or what comes
next.

They are built into the engine: a game's page uses one by name
(`<PuzzleGoals .../>`) and loading the page loads it, with the `StarBar` the
summary uses. `og ui eject puzzle/goals` copies one into `assets/ui/puzzle/`
to change it; the game's copy of the same name wins.

## Structure and appearance

Each member is HTML with component CSS in the `og-components` layer.
`:host` owns the root dimensions; shared `data-layout` and `data-gap`
primitives arrange the interior. Component CSS keeps sizes and placement.
Optional pieces use `{#if}`. Each part wears at most one shared role; the goals bar and the booster tray
are dressed by the `--play-*` role tokens (the counter's well, a goal's
socket and check, the star bar, the booster socket, its armed ring and an
out booster's gold tag), the path and summary by the `--select-*` ones. A style pack
(`og-pack`, above it) can override the named parts (docs/ui.md §3.2),
qualified by states such as `PuzzleLevelPath::node[data-state=current]`.

Hooks for a pack: `data-state` (locked, open, current, done) on level-path
lanes, trails, nodes and stars, `aria-selected` on the node the player
picked, `.star.lit` on earned stars and `data-lit` on the goals' star marks; `data-low` on the goals frame, counter
and figure, `data-kind` (moves, time) on the counter, `data-done` on goals;
`data-available`, `data-armed` and `data-out` on boosters (also
`aria-pressed` and `:disabled`), `data-out` on their counts and
`data-armed` on their state word (true only on an available, armed booster; an unavailable one
reads Unavailable); `data-result` (win, lose) on the summary's frame, band and stars,
`.primary` on its main action.

Every state also has a mark or a word, not a colour alone: a lock and
"locked" in the name of a locked level, the stars and "complete" on a done
one, a Play badge on the current one; a check on a met goal, "Running low"
on a tag under a low counter; "Unavailable" on a tag over a booster the game
refuses and a gold + where the count was when none are left,
"Armed" on the armed one; "New best" and the stars in words on the summary.
The documents animate nothing; a low counter's pulse is a pack's.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `PuzzleLevelPath` (`level-path`) | `chapters` keyed `{ id, label, earned, total }`, `levels` keyed `{ id, chapter, number, state (locked, open, current, done), stars, maxStars, x 0–1 }`, `selected` | `choose` (a level's key; a locked level is disabled), `details` (X, pad Y on the focused level) | `frame`, `chapter`, `title`, `badge`, `meter`, `lane`, `trail`, `node`, `number`, `stars`, `here`, `empty` |
| `PuzzleGoals` (`goals`) | `counter` (moves, time), `moves`, `seconds`, `low`, `score`, `progress` (toward the top star, 0–1; negative hides the bar), `stars` keyed `{ id, at 0–1 }`, `goals` keyed `{ id, icon, label, left, done }` | none | `frame`, `counter`, `label`, `figure`, `warning`, `goals`, `goal`, `socket`, `count`, `name`, `done`, `glyph`, `score`, `track`, `fill`, `star` |
| `PuzzleBoosters` (`boosters`) | `boosters` keyed `{ id, icon, name, count, available }`, `armed` | `choose` (a booster's key; an unavailable one is disabled and sends nothing) | `frame`, `booster`, `socket`, `count`, `name`, `state`, `empty` |
| `PuzzleSummary` (`summary`) | `result` (win, lose), `level`, `score`, `scoreMax`, `thresholds` keyed `{ id, at 0–1 }`, `stars`, `maxStars`, `record`, `rows` keyed `{ id, label, value }`, `total`, `canNext` | `next` (only when `canNext`), `retry`, `menu` (the Map button, Escape, pad B) | `frame`, `band`, `title`, `level`, `stars`, `earned`, `record`, `breakdown`, `row`, `rowLabel`, `rowValue`, `total`, `actions`, `next`, `retry`, `menu` |

Parts are qualified by the member (`PuzzleLevelPath.node`). A level's place
across the path (`x`) is the game's layout; the summary's star bar is the
shared `StarBar`, lit from the thresholds the game's star rules place, while
the stars awarded (`stars`) are the game's word and are what the summary
says. Booster `choose` means "the player asks for this one": the game spends
it, arms it (then passes `armed`) or refuses.

Every member carries a `fixture:default` and the states that apply to it:
level path empty, selected, locked, overflow; goals time, low, lowTime,
done, empty, overflow; boosters selected (armed), disabled, empty,
overflow; summary full (three stars, new best), lose, empty, overflow. The
goals wrap under the counter and score, and the level path and the booster
tray tighten their sizes, by the component's own width (container queries,
so a narrow panel on a wide screen gets them too); the booster tray
scrolls and the summary's breakdown scrolls under a fixed band, stars and
actions. Only the summary's padding follows the screen's width.

## Gallery and tests

`bun packages/engine/src/node/gallery.ts --only puzzle/` renders every member's fixtures under every pack (docs/ui.md §6.5).

`bun test tests/puzzle.test.ts` (from `packages/engine`) mounts every member
by name from a game page from every fixture at desktop and phone, and checks
the intents and their keys, that locked levels and unavailable boosters are
disabled and silent, keyed selection and the armed booster through a
reorder, keyboard and pad focus, the details action (never for a locked
level), and the words that carry each state.

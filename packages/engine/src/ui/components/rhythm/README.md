# rhythm: built-in components

Song select, difficulty, the judgement call, the score HUD and the results
screen, as documents over core controls (docs/ui.md §4). They draw what the
game tells them and return intents with stable row keys. They never judge a
hit, keep time, count a combo, compute a score, accuracy, life, grade or
verdict, or decide what is unlocked: every figure arrives from the game,
formatted where it is text.

They are built into the engine: a game's page uses one by name
(`<RhythmJudgement judgement={$judge} hit={$hit} combo={$combo} />`) and
loading the page loads it, with the `Spinner` and `ProgressBar` it uses.
`og ui eject rhythm/results` copies one into `assets/ui/rhythm/` to change
it; the game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing from
tokens. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`) supplies every appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `RhythmResults::row[data-judgement=miss]` or
`button::part(RhythmDifficulty\.row)[data-tier=expert]`. The fantasy pack's
dress is `templates/styles/fantasy/rhythm.css`.

Hooks for a pack: `data-status` (``, loading, error) on the song select
frame; `data-locked`, `data-fresh`, `data-cleared`, `data-previewing` and
`aria-selected` on song rows, `data-tag` (selected, previewing, fresh,
cleared, locked) on their badges; `data-tier` (easy, normal, hard, expert,
master, special), `data-locked` and `aria-selected` on difficulty tiles,
`data-tag` (grade, locked, selected) on their badges; `data-judgement` on
the judgement frame and plate, `data-timing` on the plate and its badge,
`data-broken` on its combo;
`data-danger` and `data-failed` on the score HUD frame and life row,
`data-tag` (danger, failed) on its badges; `data-cleared` and `data-mark`
(fullCombo, allPerfect) on the results frame, `data-tag` (cleared, failed,
fullCombo, allPerfect, newBest) on its badges, `data-judgement` on tally
rows. A state is always also a word (New, Cleared, Locked, Selected,
Previewing, Not played, Perfect…Miss, Early, Late, Combo broken, Danger,
Failed, New best, Full combo, All perfect), never colour alone; a tier's
colour sits beside its label.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `RhythmSongSelect` (`song-select.svelte`) | `title` (empty: Songs), `songs` keyed `{ id, title, artist, bpm, length, jacket, locked, fresh, cleared }`, `selected`, `previewing`, `status` (``, loading, error) | `choose` (row key), `preview` (Preview: the selected song's key; X / pad Y on a focused row: that row's key); a locked row and its Preview are disabled and silent; `retry` (Retry, offered only while `status` is error) | `frame`, `band`, `title`, `row`, `name`, `badge`, `jacket`, `detail`, `cell` |
| `RhythmDifficulty` (`difficulty.svelte`) | `title` (empty: Difficulty), `levels` keyed `{ id, tier, label, rating, grade, score, locked }`, `selected` | `choose` (tile key; a locked chart is disabled and silent) | `frame`, `band`, `title`, `row`, `name`, `cell`, `badge` |
| `RhythmJudgement` (`judgement.svelte`) | `judgement` (perfect, great, good, bad, miss, none), `label` (the game's word; empty: the kit's), `timing` (early, late, empty), `hit` (the hit's id: a new one draws the plate afresh, replaying a pack's entrance animation), `combo` (0 hides), `broken` | none | `frame`, `plate`, `title`, `badge`, `combo` |
| `RhythmScoreHud` (`score-hud.svelte`) | `score`, `accuracy` (formatted; empty hides), `life` (0 to 1; negative hides), `danger`, `failed`, `progress` (0 to 1, a `ProgressBar`) | none | `frame`, `cell`, `title`, `figure`, `row`, `badge` |
| `RhythmResults` (`results.svelte`) | `song` (empty: Results), `grade` (empty hides), `cleared`, `score`, `previous` (0 hides), `newBest`, `maxCombo`, `accuracy`, `mark` (fullCombo, allPerfect, empty), `counts` keyed `{ id, judgement, label, count }`, `canRetry` | `retry` (silent when `canRetry` is false), `next` (button, Escape, pad B) | `frame`, `band`, `title`, `grade`, `badge`, `cell`, `figure`, `row`, `name` |

`{#key hit}` draws the judgement's plate afresh for each hit; the documents
set no motion of their own, so the pop is the pack's, on
`RhythmJudgement.plate` (the fantasy pack zooms it in on the motion tokens;
under reduced motion it does not zoom and the word fades in quickly
instead). The judgement and the score HUD are not live regions (they change
many times a second); the empty texts and the song select's loading and
error states are. Narrow layouts come from the members' own queries:
RhythmSongSelect and RhythmResults ask their own width (container queries,
so a side column on a wide screen narrows them too), the song select
putting the selected song's details over the list and the results stacking
in one column with the actions sharing the width; RhythmDifficulty (one
chart per row) and RhythmScoreHud follow a phone's viewport.

Every member carries a `fixture:default` and the states that apply (song
select: selected, disabled, empty, loading, error, overflow; difficulty:
selected, disabled, empty, overflow; judgement: great, good, miss, none,
overflow; score HUD: danger, failed, nolife, overflow; results: failed,
fullCombo, allPerfect, disabled, empty, overflow). Loading and error apply
only to the song select: the others draw data the game already has.

## Review sheet and tests

`og ui sheet rhythm` renders `packages/engine/sheets/rhythm/`, dressed by
the fantasy pack: `songs` (the normal page: the song list with the selected
song's details and its charts), `play` (the stress page: the judgement and
the score HUD in danger over a bright sky and dark lanes, beside the
overflow results) and `states` (every member in every fixture state).
`style:default` and `hit:<judgement>:<early|late|->` are sheet commands for
the default style and motion clips.

`bun test tests/rhythm.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, checks the
declared parts, the intents and their keys (Retry on a failed load among
them), the disabled states (a locked song's row, X on it and its Preview
are silent), keyed selection through a reorder, keyboard and pad focus, Escape and pad B, the
judgement's replay, and that each state is a word.

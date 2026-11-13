# arena: built-in components

Character selection, the ban and pick draft, the versus reveal, the team
scoreboard, the round HUD and the lobby, as documents over core controls
(docs/ui.md §4). They draw what the game tells them and return intents with
stable row keys. They never compute scores, kills, ranks, round outcomes,
turn order, timers, pick legality or readiness: every figure arrives from
the game, every decision is the game's. Arena owns team and match scores;
the shooter group does not duplicate them.

They are built into the engine: a game's page uses one by name
(`<ArenaRoster characters={$characters} selected={$selected} players={$players} title="" />`)
and loading the page loads it, with the `HudTimer` the draft and the round
HUD use. `og ui eject arena/roster` copies one into `assets/ui/arena/` to
change it; the game's copy of the same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing from
tokens. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`) supplies every appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `[part="ArenaScoreboard.row"][data-local=true]`
or `button::part(ArenaRoster\.cell)[data-taken=true]`. The fantasy pack's
dress is `templates/styles/fantasy/arena.css`.

Hooks for a pack: `data-role`, `data-locked`, `data-taken` and
`aria-selected` on roster cells, `data-role` on their role,
`data-slot` (1–4) and `data-ready` on a player's mark, `data-tag` (locked,
taken, selected) on the badge; `data-phase` (ban, pick, done) on the
draft's phase badge, `aria-current` on the team column choosing and
`data-team` on its heading, `data-kind` (ban, pick) and `data-state`
(empty, filled, banned, active) on a slot, `data-state` on its portrait,
`data-tag` (banned, banning, picking, open) on its badge; `data-team` on
versus sides and headings, `data-local` on its cards; `data-team` on the
scoreboard's headings and team scores, `data-local` and
`data-disconnected` on its rows, `data-tag` (you, disconnected) on the
badges; `data-local` on round HUD team plates, `data-team` on its scores,
`data-outcome` (won, lost, draw) on its banner; `data-open` and
`data-local` on lobby rows, `data-open` on their portrait, `data-tag`
(host, you) on their badges and `data-ready` on their state cell. A value
the kit does not know (a phase, a turn, an outcome) shows no word at all
rather than another state's. Team colours are the pack's
`--team1` / `--team2`. A state is always also a word (Locked, Taken by …,
Selected, P1…P4 with a check, Ban phase, Pick phase, Draft complete,
Banned, Banning, Picking, Open, You, Disconnected, Round won, Round lost,
Round drawn, Host, Ready, Not ready, Open slot), never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `ArenaRoster` (`roster.svelte`) | `title` (empty: Choose your champion), `characters` keyed `{ id, name, portrait, icon, role, roleLabel, locked, takenBy }`, `selected`, `players` keyed `{ id, slot (1–4), at (a character id), ready }` | `choose` (the cell's key; silent for locked and taken cells, which are disabled) | `frame`, `band`, `title`, `cell`, `portrait`, `name`, `role`, `badge`, `mark` |
| `ArenaDraft` (`draft.svelte`) | `title` (empty: Draft), `phase` (ban, pick, done), `active` (a, b, empty), `nameA`, `nameB`, `slots` keyed `{ id, team, kind (ban, pick), name, portrait, icon, player, active }`, `timeLeft` (negative hides the `HudTimer`), `timeTotal`, `canConfirm` | `confirm` (Lock in, also the details action: X / pad Y; silent when `canConfirm` is false; hidden when done) | `frame`, `band`, `title`, `team`, `heading`, `slot`, `portrait`, `name`, `badge` |
| `ArenaVersus` (`versus.svelte`) | `nameA`, `nameB`, `sideA`, `sideB` keyed `{ id, name, portrait, icon, character, rank, local }`, `at` (the reveal's id; a new one replays the entrance) | none | `frame`, `side`, `heading`, `card`, `portrait`, `name`, `rank`, `badge` |
| `ArenaScoreboard` (`scoreboard.svelte`) | `title` (empty: Scoreboard), `columns` (kda, score), `nameA`, `nameB`, `scoreA`, `scoreB` (formatted; empty hides), `rows` keyed `{ id, team (a, b), name, kills, deaths, assists, score, local, disconnected }` in the game's order | none | `frame`, `band`, `title`, `team`, `heading`, `score`, `row`, `cell`, `name`, `badge` |
| `ArenaRoundHud` (`round-hud.svelte`) | `round`, `rounds` (0 hides the total), `nameA`, `nameB`, `scoreA`, `scoreB`, `localTeam` (a, b, empty), `timeLeft` (negative hides the `HudTimer`), `timeTotal`, `outcome` (won, lost, draw, none), `at` (the banner's id), `detail` | none | `frame`, `team`, `score`, `round`, `banner`, `title`, `badge` |
| `ArenaLobby` (`lobby.svelte`) | `title` (empty: Lobby), `detail`, `slots` keyed `{ id, name, portrait, icon, open, host, ready, local }`, `canInvite`, `isReady`, `canReady`, `isHost`, `canStart` | `invite` (an open seat's key), `ready` / `unready` (the button, also the details action: X / pad Y; silent when `canReady` is false), `start` (host only; silent when `canStart` is false) | `frame`, `band`, `title`, `row`, `portrait`, `name`, `badge`, `cell` |

The roster's cells are a listbox: the pad and the arrows walk the free
cells (disabled ones are skipped) and confirm chooses. The versus and the
round banner animate on motion tokens and, under reduced motion, do not
play (they are simply there); the banner is a status region (`aria-live`
assertive). The roster, the draft, the scoreboard and the lobby sit in
panels and answer their own width (a container query on the frame: under
600px, the roster under 380px): the roster's cells shrink, the draft's
columns tighten, the
scoreboard puts each name on its own line above its figures (in even
columns), the lobby's actions share the width. The versus fills the
screen and the round HUD spans it, so they answer the screen's width: the
versus stacks its sides around the seal, the round HUD puts each team's
name on a line of its own over its score.

Every member carries a `fixture:default` and the states that apply
(roster: empty, none, full, disabled, overflow; draft: ban, empty, done,
overflow; versus: teams, waiting, overflow; scoreboard: score, empty,
disconnected, overflow; round HUD: won, lost, draw, critical, overflow;
lobby: full, guest, empty, disabled, overflow). Loading and error do not
apply: the members draw data the game already has (a game shows a waiting
lobby or roster through `detail` and the empty states).

## Review sheet and tests

`og ui sheet arena` renders `packages/engine/sheets/arena/`: `select` (the
normal page: roster, draft and lobby), `match` (the stress page: the round
HUD with its won banner over a bright sky and a dark floor, the ten-player
scoreboard and the five-a-side versus with long names) and `states` (every
member in every fixture state), in the fantasy pack, and `select` and
`match` in the core default style.

`bun test tests/arena.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, and under
the default style; checks the declared parts, the intents and their keys,
the disabled states, keyed selection through a reorder, keyboard and pad
focus, the team placement and order, and that each state is a word.

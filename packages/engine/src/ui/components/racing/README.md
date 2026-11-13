# racing: built-in components

Telemetry, laps and standings, the race banner, garage selection and the
results screen, as documents over core controls (docs/ui.md §4). They draw
what the game tells them and return intents with stable row keys. They never
compute speed, gear, lap times, gaps, positions, points, prices or unlocks:
every figure and time arrives formatted from the game.

They are built into the engine: a game's page uses one by name
(`<RacingStandings rows={$rows} />`) and loading the page loads it, with the
`StatBar` and `HudTimer` it uses. `og ui eject racing/garage`
copies one into `assets/ui/racing/` to change it; the game's copy of the
same name wins.

## Structure and appearance

Each member's `<style>` holds structure only: layout, sizes and spacing from
tokens. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`) supplies every appearance through the
members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `RacingStandings::row[data-player=true]` or
`button[data-locked=true]::part(RacingGarage\.tile)`. The fantasy pack's
dress is `templates/styles/fantasy/racing.css`.

Hooks for a pack: `data-dial` (radial, bar), `data-shift`, `data-boosting`
and `data-redline` on the telemetry frame; `data-final` on the lap HUD and
its lap cell, `data-state` (ahead, behind, even) on the split delta,
`data-fresh` on the last-lap row; `data-player` and `data-state` (racing,
pitting, retired, finished) on standings rows and their badge; `data-kind`
(countdown, go, finalLap, finish, none) on the banner frame and plate;
`data-owned`, `data-locked`, `data-chosen` on garage tiles and the hero and `data-tag`
(driving, locked, owned, price) on their badge; `data-player`, `data-state`
(finished, dnf, dsq) and `data-fastest` on result rows. A state is always
also a word (You, Pit, Out, Fin, Ahead, Behind, Final lap, Best lap,
Fastest lap, DNF, DSQ, Locked, Too costly, Driving), never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `RacingTelemetry` (`telemetry.svelte`) | `speed`, `units` (the game's word: km/h, mph), `gear` (R, N, 1…; empty hides), `rpm`, `rpmMax`, `redline` (0 none), `shift`, `boost`, `boostMax` (0 hides), `boosting`, `dial` (radial, bar) | none | `frame`, `dial`, `speed`, `units`, `gear`, `badge` |
| `RacingLapHud` (`lap-hud.svelte`) | `lap`, `laps` (0 hides the total), `position`, `racers` (0 hides), `current`, `best`, `last` (formatted times; empty shows a blank time), `split` (formatted delta; empty hides), `splitState` (ahead, behind, even), `freshBest`, `finalLap`, `timeLeft` (negative hides the `HudTimer`), `timeTotal`, `warnAt` (seconds left when the timer turns warn; optional, 10) | none | `frame`, `cell`, `title`, `row`, `delta`, `badge` |
| `RacingStandings` (`standings.svelte`) | `title` (empty: Standings), `rows` keyed `{ id, position, name, gap, player, state }` in the game's order | none | `frame`, `band`, `title`, `row`, `cell`, `name`, `gap`, `badge` |
| `RacingBanner` (`banner.svelte`) | `kind` (countdown, go, finalLap, finish; none or any other word shows nothing), `figure` (the count), `at` (the call's id; a new one replays the entrance), `place` (finish ordinal; 0 hides), `detail` | none | `frame`, `plate`, `title`, `badge` |
| `RacingGarage` (`garage.svelte`) | `title` (empty: Garage), `vehicles` keyed `{ id, title, body, image, owned, locked, affordable, price }`, `shown`, `index` (the shown car's place, read as n of N), `chosen`, `stats` keyed `{ id, label, value, max, text }` | `choose` (tile key), `select` (Drive, the shown car's key), `buy` (Buy, the shown car's key; silent when the car is not `affordable`), `prev`/`next` (the arrows, Q/E, bumpers), `back` (button, Escape, B) | `frame`, `band`, `title`, `hero`, `tile`, `name`, `badge`, `row` |
| `RacingResults` (`results.svelte`) | `title` (empty: Results), `rows` keyed `{ id, position, name, time, points, fastest, player, state }`, `bestLap`, `bestBy`, `canRetry` | `continue`, `retry` (button or the details action, X / pad Y; silent when `canRetry` is false) | `frame`, `band`, `title`, `heading`, `row`, `cell`, `name`, `badge` |

The garage shows one action for the shown car: Drive when owned, Driving
(disabled) when it is the car driven, Buy with its price when it can be had
(disabled and named Too costly when the row's `affordable` is false),
Locked (disabled, the reason beside it) when it cannot. The game decides all
of them. The banner is a status region (`role="status"`, polite); its
entrance is CSS animation on motion tokens and, under reduced motion, does
not play (the call is simply there). The garage and the results lay out by
their own width (container queries on the frame): in a narrow panel the
garage stacks the shown car over the ratings and the action and the results'
actions share the width. The telemetry dial shrinks by a media query: its
frame is sized by its contents, so it has no width of its own to query.

Every member carries a `fixture:default` and the states that apply
(telemetry: redline, bar, reverse, idle, overflow; lap HUD: grid,
finalLap, checkpoint, overflow; standings: empty, single, finished,
overflow; banner: go, finalLap, finish, none, overflow; garage: selected,
buy, short, locked, empty, overflow; results: empty, disabled, overflow). Loading
and error do not apply: the members draw data the game already has.

## Review sheet and tests

`og ui sheet racing` renders `packages/engine/sheets/racing/`, dressed by
the fantasy pack: `race` (the normal page: telemetry, lap HUD, standings and
the start banner over a bright sky and a dark track), `garage` (the stress
page: a ten-car garage with long names and a fourteen-row classification)
and `states` (every member in every fixture state). `style:default` and
`call:<kind>:<n>` are sheet commands for the default style and motion clips.

`bun test tests/racing.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, checks the
declared parts, the intents and their keys, the disabled states, keyed
selection through a reorder, keyboard and pad focus, and that each state is
a word.

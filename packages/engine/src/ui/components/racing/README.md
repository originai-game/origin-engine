# racing: built-in components

The race HUD (the place and lap times, the position tower, the
tachometer, the track map, the boost meter, the pedals, the car's tyres and
fuel, the calls and the points earned), garage selection and the results
screen, as documents over core controls (docs/ui.md §4). They draw what the
game tells them and return intents with stable row keys. They never
compute speed, gear, lap times, gaps, positions, points, prices or unlocks:
every figure and time arrives formatted from the game.

They are built into the engine: a game's page uses one by name
(`<RacingStandings rows={$rows} />`) and loading the page loads it, with the
`StatBar` and `HudTimer` it uses. `og ui eject racing/garage`
copies one into `assets/ui/racing/` to change it; the game's copy of the
same name wins.

## Structure and appearance

Each member's `<style>` holds layout only. The HUD members (telemetry, lap
HUD, standings, banner, track map, boost, pedals, car, bonus) stand bare
over the scene in the default style's smoked glass and lit edges: every
colour, fill, rim, radius and type step is a `--hud-*` or `--bar-*` role
token (a racing set beside the shooter's and the RPG's: the player's row,
a place's box, the best lap's violet, the rev arc, its red zone and the
dial's face, the shift lights' ramp, the track's road and line, and the
place, time and speed type steps), and every type step is in font steps,
so it follows the player's text scale. The HUD's accent (`--hud-accent`)
lights the player's row, the lap plate, the gear, the lap driven on the map
and the slipstream; a game sets it, with the tokens made from it, to its
key colour. Driver, track and meter names stay on one line and scroll when
they overflow; a row gives the name its free space first. The garage and
the results are dressed by a style pack through their parts, as before; the
fantasy pack dresses them through its tokens alone
(`templates/styles/fantasy/base.css`).

Hooks for a pack: `data-shift` and `data-redline` on the telemetry frame;
`data-final` on the lap HUD and its lap plate, `data-trend` (up, down) on the
places gained, `data-fresh` on the last-lap row and `data-state` (ahead,
behind, even) on the split row and its delta; `data-player` and
`data-state` (racing, pitting, retired, finished) on standings rows;
`data-kind` (countdown, go, finalLap, finish, call, none) on the banner
frame and plate; `data-firing` on the boost frame; `data-pedal` (throttle,
brake) on a pedal; `data-state` (good, worn, bad) on a tyre and `data-low`
and `data-pit` on the car's frame; `data-owned`, `data-locked`, `data-chosen`
on garage tiles and the hero and `data-tag` (driving, locked, owned, price)
on their badge; `data-player`, `data-state` (finished, dnf, dsq) and
`data-fastest` on result rows. A state is always also a word (Leader, Pit,
DNF, Fin, Up 2 places, Final lap, Best lap, Shift, Fuel low, Box this lap,
Fastest lap, DSQ, Locked, Too costly, Driving), never colour alone.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `RacingTelemetry` (`telemetry.svelte`) | `speed`, `units` (the game's word: km/h, mph), `gear` (R, N, 1…; empty hides), `mode` (Auto, Manual; empty hides), `rpm`, `rpmMax`, `redline` (0 none), `scale` keyed `{ id, value, label }` (the dial's figures), `lights` (0 to 1; negative hides), `shift` | none | `frame`, `dial`, `track`, `fill`, `redline`, `tick`, `mark`, `face`, `lights`, `speed`, `units`, `gear`, `mode`, `badge` |
| `RacingLapHud` (`lap-hud.svelte`) | `lap`, `laps` (0 hides the total), `position`, `racers` (0 hides the place), `gained` (places; 0 hides), `current`, `best`, `last` (formatted times; empty shows a blank time), `sector`, `split` (formatted delta; empty hides the row), `splitState` (ahead, behind, even), `freshBest`, `finalLap`, `timeLeft` (negative hides the `HudTimer`), `timeTotal`, `warnAt` (optional, 10) | none | `frame`, `place`, `of`, `lap`, `gained`, `times`, `row`, `label`, `time`, `delta`, `badge` |
| `RacingStandings` (`standings.svelte`) | `title` (read, not drawn; empty: Standings), `rows` keyed `{ id, position, name, gap, color, player, state }` in the game's order | none | `frame`, `row`, `place`, `car`, `name`, `gap`, `badge` |
| `RacingBanner` (`banner.svelte`) | `kind` (countdown, go, finalLap, finish, call; none or any other word shows nothing), `figure` (the count), `text` (the call's words for call), `at` (the call's id; a new one replays the entrance), `place` (finish ordinal; 0 hides), `detail` | none | `frame`, `plate`, `title`, `badge`, `caption` |
| `RacingTrackMap` (`track-map.svelte`) | `path`, `driven` (SVG paths in the map's 260 × 220 box), `startX`, `startY`, `startAngle`, `cars` keyed `{ id, x, y, heading, color, player, name }`, `name`, `length` | none | `frame`, `map`, `road`, `line`, `driven`, `start`, `car`, `self`, `name`, `length` |
| `RacingBoost` (`boost.svelte`) | `label` (empty: Boost), `icon` (texture; 0 hides), `key` (empty for none), `value`, `max`, `cells`, `firing` | none | `frame`, `icon`, `label`, `key`, `bar`, `fill` |
| `RacingPedals` (`pedals.svelte`) | `throttle`, `brake` (0 to 1) | none | `frame`, `pedal`, `well`, `fill`, `label` |
| `RacingCar` (`car.svelte`) | `tyres` keyed `{ id, label, text, state }` (good, worn, bad), `fuel` (0 to 1; negative hides), `fuelText`, `fuelLow`, `pit` | none | `frame`, `title`, `tyre`, `fuel`, `bar`, `fill`, `reading`, `badge` |
| `RacingBonus` (`bonus.svelte`) | `meter` (empty hides), `value` (0 to 1), `points` (empty hides), `detail`, `at` (a new one replays the points' entrance) | none | `frame`, `meter`, `label`, `bar`, `fill`, `points`, `detail` |
| `RacingGarage` (`garage.svelte`) | `title` (empty: Garage), `vehicles` keyed `{ id, title, body, image, owned, locked, affordable, price }`, `shown`, `index` (the shown car's place, read as n of N), `chosen`, `stats` keyed `{ id, label, value, max, text }` | `choose` (tile key), `select` (Drive, the shown car's key), `buy` (Buy, the shown car's key; silent when the car is not `affordable`), `prev`/`next` (the arrows, Q/E, bumpers), `back` (button, Escape, B) | `frame`, `band`, `title`, `hero`, `tile`, `name`, `badge`, `row` |
| `RacingResults` (`results.svelte`) | `title` (empty: Results), `rows` keyed `{ id, position, name, time, points, fastest, player, state }`, `bestLap`, `bestBy`, `canRetry` | `continue`, `retry` (button or the details action, X / pad Y; silent when `canRetry` is false) | `frame`, `band`, `title`, `heading`, `row`, `place`, `name`, `badge` |

The garage shows one action for the shown car: Drive when owned, Driving
(disabled) when it is the car driven, Buy with its price when it can be had
(disabled and named Too costly when the row's `affordable` is false),
Locked (disabled, the reason beside it) when it cannot. The game decides all
of them. The banner is a status region (`role="status"`, polite); its
entrance and the bonus points' are CSS animation on motion tokens and,
under reduced motion, do not play (the call is simply there). The garage
and the results lay out by their own width (container queries on the
frame). The HUD members lay out by the screen: on a phone (narrower than
600 px) the tower keeps the player's row and the rows either side of it,
the lap HUD leaves out the last lap and sets the best lap and the delta on
one line, the tachometer shrinks and leaves out its figures and the
gearbox's mode, the track map shrinks without its name line, the bonus is
one compact row, and key caps hide under a touch screen. A page may give the tower a height as a size
container (`container-type: size`): when that height is short of the
rows (at a large text scale) the tower keeps the leader and the rows
either side of the player, then the leader and the player, so no row is
cut.

Every member carries a `fixture:default` and the states that apply
(telemetry: redline, pit, reverse, idle, overflow; lap HUD: leading, last,
grid, checkpoint, overflow; standings: leading, last (a pitting and a
retired car), empty, single, finished, overflow (long names); banner:
countdown, go, finalLap, finish, none, overflow; track map: empty, start,
overflow; boost: firing, empty, overflow; pedals: braking, empty,
overflow; car: fuel, fuelLow (with Box this lap), empty, overflow; bonus:
meter, points, empty, overflow; garage: selected, buy, short, locked,
empty, overflow; results: empty, disabled, overflow). Loading and error do
not apply: the members draw data the game already has.

## Review sheet and tests

`og ui sheet racing` renders `packages/engine/sheets/racing/`: `race`
(the normal page, the HUD laid out as its approved mockup over sample
stage art: the place, the lap and the times over the position tower top
left, the call and the slipstream with the points earned top centre, the
track map top right over the tachometer bottom right, the pedals, the
tyres and the nitro meter along the foot, and the road ahead left clear;
on a phone every readout but the foot's stands in rows at the top), `garage`
(the stress page: a ten-car garage and a fourteen-row classification) and
`states` (every member in every fixture state), dressed by the fantasy pack
or, with `style:default`, the default style. `call:<kind>:<n>` steps the
banner for motion clips.
`og ui review racing --style scifi` (or `--style cartoon`) renders the same
pages in that pack, dressed by its tokens
(`templates/styles/<pack>/base.css`).

`bun test tests/racing.test.ts` (from `packages/engine`) mounts every member
by name from a game page in every fixture at desktop and phone, checks the
declared parts, the intents and their keys, the disabled states, keyed
selection through a reorder, keyboard and pad focus, and that each state is
a word; the HUD members also mount from every fixture in the default style.

## Document layout

The members use HTML with CSS and `:host` for their roots. Optional badges,
figures and readouts use `{#if}`; scroll regions are HTML boxes with
overflow. The tachometer places its figures and ticks round the arc with
`sin()` and `cos()` of each one's share of the rev range; the shift lights
are one ramp clipped to the share lit and cut into cells by a mask. The
standings' rows carry the car's colour as `--hud-car`, so a car without
one keeps the token's neutral stripe.

The sheet declares `race-default`, `garage-default` and `states-default`
and supplies the default style's font explicitly, as it does for the
pack's icons. Pass each page's commands when using `og ui preview`.

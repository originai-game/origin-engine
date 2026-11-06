# Redline Ring — the racing archetype (3D)

Six cars, three laps of an 810 m circuit on a clear afternoon: a start
straight past the grandstand, a fast right sweeper, the back esses, a
hairpin, a long left and the run back to the gantry. Hold Throttle, steer
with the stick, brake into the bends; Brake held with Throttle through a
bend at speed drifts, and a drift fills the boost meter that Boost spends.
Pass both checkpoints and cross the line three times. First across the line
wins; every other place is a result with its points; a seat still running
30 s after the winner is out (DNF). Then the results, and again.

红线赛道：六辆车，三圈 810 米的赛道，晴朗的午后：看台前的发车直道、一个高速右弯、后段的连续弯、一个发夹弯、一个长左弯，再冲回起点龙门。按住油门，用摇杆转向，入弯前刹车；在弯中高速时同时按住刹车和油门就会漂移，漂移会充满氮气槽，按氮气消耗它。依次通过两个检查点并三次越过终点线。第一个冲线者获胜；其他名次是带积分的成绩；冠军冲线 30 秒后仍未完赛的玩家算作未完赛（DNF）。随后是成绩榜，再来一场。

This is a complete Origin Engine project: `og run .` plays it, `og build .`
makes the play page, `og test .` runs its playtests and `og polish .`
scores them.

```sh
og run . --headless --commands "0.3:race:go;0.4:key_down:KeyW;4:screenshot:drive.png;4.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships (behind "tap to play")
bun tools/make-assets.ts     # assets/audio/*.wav again, byte for byte
```

## What it demonstrates, and the skills it exercises

- **A circuit made of numbers** (`src/track.ts`, `src/mesh.ts`): a closed
  centripetal Catmull-Rom spline through 21 points, resampled every 2 m
  with its tangent, right and curvature; the road (a rubbered racing line,
  kerbs on the bends, the chequered line, the grid boxes), red and white
  barriers, the grandstand and its crowd, the start gantry with its five
  lamps, checkpoint gates, trees and faceted hills are flat-shaded,
  vertex-coloured `og.render.bufferGeometry` meshes built in `start`.
  The racing line is an elastic band relaxed inside the road (wide in,
  the apex, wide out) with a speed profile from its curvature and the
  braking the next bend needs.
- **Arcade handling on the fixed tick** (`src/car.ts`): grip that catches
  a slide at `Tuning.grip`, a drift that lets it hang (`driftGrip`) and
  turns harder (`driftTurn`), a boost meter, barriers that push a car back
  on the road and turn it along it, capsule contacts between cars that
  trade their speed. Pure arithmetic on the cars' rows: no physics
  engine, nothing read from a clock, the same on every peer.
- **Rivals on the racing line**: pure pursuit of a point ahead on the
  line, each at its own share of `Tuning.rivalPace`, a lane change to pass
  a slower car, a follow cap behind one it cannot pass and a mild elastic
  (a rival far ahead of every seat eases off, one far behind presses).
- **The racing kit's HUD** (`@origingame/engine/extensions/racing`): the
  lap timer's head (place and lap, the sector pips) and its times (the
  running lap, best, last, the split at each checkpoint), the standings
  tower, the arcade speedometer with the boost meter in four segments, and
  the race banner (the countdown's 3 2 1 GO, a checkpoint's split, the
  final lap, wrong way, the finish); og.ui's minimap of the circuit beside
  them; the kit's results document (`assets/ui/racing/race-results.ogt`).
- **Three views** (`src/view.ts`, the 3D starters' shared rig): a chase
  camera behind the car (third-person-3d), the bonnet camera
  (first-person-3d) and a fixed-angle orthographic view from above
  (isometric-2.5d). `CarView` writes each seat's look from its car's
  heading before `ViewRig` points the camera: the chase heading eases
  after the car's, so a drift swings the car across the frame.
- **Feel** (og-feel): the body rolls out of a slide, squats under power
  and boost and dives under the brake; the front wheels steer; a flame at
  the exhaust while boosting; tyre smoke while drifting; sparks, a flash
  and a shake on a contact; a punch when the boost lights; the field of
  view widening with speed; the gantry's lamps through the countdown.
- **Sound** (og-audio): an engine loop per seat pitched through five
  gears, a tyre loop faded in with the slide, the rivals' engines heard
  where they are (in a room every seat's engine, tyres and cues sound on
  its car alike on every peer, since sources are hashed world state, and
  each device hears them through its own camera), a voice per event (the countdown, contacts, checkpoints,
  laps, a best lap, the boost, the finish), "Redline" on the music bus
  ducked under the effects, a limiter on the master.
- **Screens** (og-ui): the title over the field lapping on the autopilot,
  seen from broadcast camera posts along the circuit (cutting from post to
  post as the leader passes),
  the pause and the results as screen documents, the settings dialog, a
  dissolve between screens (a fade with reduced motion), the broadcast
  style pack as `assets/styles/redline`.
- **Languages** (og-i18n): every string is a key of `assets/strings/en.ogt`
  and `zh-CN.ogt`.
- **The platform** (og-platform-sdk): settings through
  `loadSettings`/`saveSettings`; the record (`{races, wins, bestLap}`) read
  on the first ticks and written at the revision read, merged when a write
  is refused as stale; a `race.over` telemetry event.

## Seats: alone, two on one device, two over a room

Every seat drives a car (`src/seats.ts`, the same file in every starter;
`src/creation.ts` holds the creation choices and the controls): the game
reads `og.PlayerInput` alone, so the race plays

- **alone**: seat 0 from both key sets, any pad, and on a touch screen the
  game's own controls (Left and Right bottom left; Brake, Boost, whose
  ring is the meter, and Throttle bottom right);
- **local-multiplayer**: two cars on one device, WASD, Space (pad 1, the
  left set of touch controls) against the arrows and Enter (pad 2, the
  right set). The screen splits in halves (an `og.CameraView` per seat),
  each with its own place, lap, times and speedometer, the minimap and
  Pause between them (on a phone held sideways the dial sits beside the
  times; upright, where a half is under 200 px wide, a small plate of its
  own holds the place, lap and time);
- **online-lockstep**: two peers over a room, each camera chasing its own
  car (`og.Lockstep.localPlayer`), every peer running the whole race.

The seats' cars start at the back of the grid. Menus act through the
seats too: Race and Race again press Throttle; Retire and Title press
Menu (a button with no key, `surface` in the controls); Pause and Resume
press Pause, so a room acts on one tick on every peer. `bot:<n>` drives
this device's seats from a script (the room test).

## Rules

- **Driving.** Throttle to 42 m/s (151 km/h), Brake at 28 m/s², the stick
  steers (2.1 rad/s at full lock, at speed). Off the throttle the car
  rolls and slows.
- **Drift and boost.** Brake held with Throttle and a turn above 14 m/s
  drifts: the tail steps out and the turn tightens; it holds while both
  are held and the car keeps 9 m/s. A second of drifting fills 30 % of the
  boost meter. Boost spends 42 % a second for more push and a 52 m/s top.
- **Contacts.** A barrier stops a car at the road's edge and turns it
  along the road; two cars push apart and trade speed. Each contact
  counts (`Car.hits`, `Game.bumps`).
- **The race.** Three seconds of lights, then GO (the field stands until
  then). A lap counts only after both checkpoints in order (at a third
  and two thirds of the lap); backing over the line takes the lap back.
  The place is the finished in the order they crossed, then the rest by
  distance run.
- **The end.** First across the line after three laps wins. The race is
  over once every seat has finished, or `Tuning.closeAfter` (30 s) after
  the winner, when a seat still running is out (DNF). 2.5 s later the
  results: the place, the classification with gaps and points (10 8 6 4 2
  1), the best lap. Race again starts from the grid; Title goes back. Only
  the record carries over.

**Losing.** There is no game-over screen. A place behind the winner is a
result ("On the podium" for second and third, "Race complete" behind
them, each with its points); not finishing within the window is "Did not
finish", the only outcome that scores nothing. The record counts races
and wins.

Everything is deterministic: the tick is fixed, the handling and the
rivals are plain arithmetic on world state, and `tests/replay.og.ts`
replays a race (drifts, boost, contacts, the rivals) to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| the countdown | the gantry's lamps light one a second with a beep, all green at GO with a higher tone, the banner's 3 2 1 GO | `Race` in `src/race.ts`, `Scene` |
| full throttle | the engine climbs through five gears, the body squats, the view widens | `Voices`, `Scene` |
| a drift | the tail out, smoke, the tyres' squeal, the body rolled, the boost ring filling | `Drive`, `Scene`, `Voices` |
| boost | the exhaust flame, a punch, the view wider still, the meter draining | `Scene` |
| a contact | sparks, a flash on the body, a knock, a shake for this device's car | `Drive`, `Scene` |
| a checkpoint | the split against the best lap, green ahead or red behind, a chime | `laps` in `src/car.ts`, `banner` in `src/ui.ts` |
| the line | the lap's time, a best-lap fanfare, "Final lap" in gold | `laps`, `banner` |
| the finish | the banner's place, the finish cue, the autopilot takes the car on | `Drivers`, `banner` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, paused and held, the race's clock, the first finish and the close, bumps, sound requests, the seats' cars |
| `Tuning` | resource | the knobs below |
| `Car` | each car | its slot, seat (-1 a rival) and body; this tick's steer, throttle, brake and boost; heading, velocity, speed, slip, drifting, boosting, the meter; the chase camera's heading; contacts; where it is on the lap (sample, along, across), laps, checkpoints, progress, lap and split times, best lap, finish, out, place, wrong way; a rival's pace and lane |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |
| `SeatLook` | each seat's car | its view's heading and pitch (src/view.ts) |

The player's settings and the record are this device's (`src/device.ts`),
never the world's.

## Systems (in tick order)

| System | Job |
|---|---|
| `Seats` | each seat this device plays: its keys, pad and touch controls to `og.PlayerInput` |
| `Device` | this device's platform answers: the settings loaded, the record read and written (it writes no shared state, so it hears them in a room too) |
| `Ui` | the screens, the seats' choices (race, pause, retire, again), the hold, the results and the record, the HUD, the buses |
| `Drivers` | each seat's car from its input (the autopilot on the title and once finished) |
| `Rivals` | each rival from the racing line |
| `Drive` | the handling, the barriers, contacts between cars, the laps and checkpoints |
| `Race` | the countdown's cues, the places, the first finish, the close and DNF |
| `CarView` | each seat's look from its car |
| `ViewRig`, `og.systems.cameras` | the camera (or each half's view) on its car, the punch and shake |
| `Scene` | the bodies, wheels, flame, smoke, sparks, the field of view, the lamps |
| `Voices` | the engine and tyre loops, every requested sound once |

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `topSpeed`, `accel`, `brake` | 42 m/s, 15, 28 m/s² | straight-line pace |
| `steer`, `grip` | 2.1 rad/s, 9 /s | the turn and how fast a slide is caught |
| `driftGrip`, `driftTurn` | 1.6 /s, 1.45 | how a drift hangs and tightens |
| `boostSpeed`, `boostAccel`, `boostDrain`, `driftCharge` | 52 m/s, 12 m/s², 0.42 /s, 0.3 /s | the boost and how drifting earns it |
| `closeAfter` | 30 s | how long after the winner a seat may still finish |
| `rivalPace` | 0.93 | the rivals' speed on the line, as a share of what it allows |

## What a Maker changes first

1. **The circuit**: `CONTROL` in `src/track.ts` (keep the start straight
   along -z through the origin, and the bends' radius above 10 m), `WIDTH`,
   `LAPS`, `SECTORS`; the scenery in `src/mesh.ts`.
2. **The handling**: `Tuning`, then the drift's entry in `Drive`.
3. **The rivals**: `rivalPace`, each rival's `PACE` and the liveries in
   `src/world.ts`, the line's speed profile (`racingLine`'s arguments).
4. **The look**: the sky, fog, sun and post stack in `buildCircuit`, the
   meshes' colours in `src/mesh.ts`, the style in `assets/styles/redline`.
5. **The screens**: `assets/ui/main-menu.ogt`, `pause.ogt` and
   `racing/race-results.ogt`; a button is an element with
   `on click emit <event>`, which `src/ui.ts` answers.
6. **Words**: `assets/strings/*.ogt`.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `race[:go]` | a race from the grid now, as Race does; `go` skips the countdown |
| `warp:<along>[:<across>[:<speed>]]` | seat 0's car `along` metres past the line, `across` right of the centre, at a speed, facing down the road |
| `warp_seat:<seat>:…`, `warp_car:<slot>:…` | the same for a seat's car, or any car of the field (0 to 5, the seats' first) |
| `lap:<slot>:<n>` | a car has done n laps: the line next begins lap n + 1, or finishes after the last |
| `bot:<n>` | a scripted player drives this device's seats (0 hands the input back) |
| `viewport:<desktop\|phone\|landscape>` | lays the UI out for a phone in portrait (390 × 844 at 3×) or held sideways (844 × 390) |
| `language:<en\|zh-CN>` | switches the language |
| `stale_record` | another device writes the record, so this one's next write is refused as stale and merged |
| `settings` | opens the settings |
| core actions | `touch_move_to:Play`, `Pause`, `Throttle`, `Brake`, `Boost`, `Left`, `Right`, `Restart`, `Continue`; WASD, the arrows, Space, Enter, Escape; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the clock, the
countdown, paused, closed, bumps, each seat's car (place, laps,
checkpoint, along, across, speed, boost, drifting, boosting, best and
last lap, finish, out, contacts, position), the record and the language.
The playtests read `Game` and each car's `Car` with `get:`.

## Playtests (`og test .`)

Every playtest reads the seat count from `src/creation.ts`, so the same
scripts pass alone, two on one device and two over a room.

| File | Proves |
|---|---|
| `tests/seats.og.ts` | each device reaches its own seat's `og.PlayerInput` and drives that car only: the keys, pad 1 and 2, both sets of touch controls (Throttle, Right, Boost) |
| `tests/race.og.ts` | the countdown holds the field; a missed checkpoint times no lap, both in order then the line time one; the last lap's finish, the close, the results, the record in the store; Throttle races again |
| `tests/dnf.og.ts` | a rival wins, the seats still running are out, the results say Did not finish, the record counts no win, Title |
| `tests/drift.og.ts` | a drift fills the boost; Boost is quicker than Throttle alone; the barrier keeps the car on the road; a car driven into another pushes it on without passing through |
| `tests/rivals.og.ts` | the rivals lap the circuit from the grid, inside the barriers, never the wrong way, a lap in 20 to 40 s |
| `tests/pause.og.ts` | Escape holds the race; settings switch to 简体中文 and move the music; Done saves through og.platform; Resume; Retire |
| `tests/phone.og.ts` | 1170 × 2532: the menu and HUD inside the safe area, touch controls at least 44 px, Throttle drives and Right turns |
| `tests/frames.og.ts` | the title, the countdown, a drift and the results at 1280 × 720 match `references/<tier>/<variant>/` (the view, and the split screen with two seats) |
| `tests/frames-phone.og.ts` | a race on a phone held sideways (2532 × 1170) matches its reference |
| `tests/replay.og.ts` | a race with drifts, boost and contacts, run twice, ends on one hash |
| `tests/bots.og.ts` | scripted players drive the seats; online, two processes race one race in a room (one peer in 简体中文) and hash the same at tick 600 with no desync |

## Assets

Nothing here is downloaded. `tools/make-assets.ts` regenerates the sounds
and the music byte for byte; the circuit and every model are code; the
smoke and spark are the engine's own CC0 assets. `LICENSES.md` lists every
file.

## Known limits (engine gaps, not design)

- `src/view.ts` splits the screen left and right whatever its shape, so
  two seats on a phone in portrait each get a tall, narrow half.
- The racing kit's lap timer is 220 px wide on a phone with no width of
  its own, wider than a half of an upright phone, so that layout draws the
  place, lap and time in a plate of its own (`miniLaps` in `src/ui.ts`).
- The kit's race banner keeps the countdown's lights plate at full
  opacity through GO's second while the GO numeral fades.

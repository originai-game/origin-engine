# Meridian Loop

A night time trial on a neon circuit, on Origin Engine: three laps from a
standing start, about two minutes a run, written the way a game author
would write it, from the knowledge package's skills and wiki and the
generated types, with nothing but TypeScript, generated assets and `og`.
It is the fourth dogfood game (after the 3D top-down
[`lantern-vale`](../lantern-vale), the 2D platformer
[`hollowbell`](../hollowbell) and the third-person
[`kestrel-stair`](../kestrel-stair)), written to exercise what none of them
did: a driven car (`og.Vehicle`), a lap timer, a ghost, a replay from the
engine's determinism, the racing HUD, engine audio and skid marks. What it
found is [`FINDINGS.md`](FINDINGS.md).

After midnight the Meridian Loop's barriers glow: magenta through the
first sector, cyan through the second, amber through the third. The car
waits on the grid behind the gantry; three beeps, GO. Drive (W or ↑
throttle, S or ↓ brake and reverse, A D or ← → steer, Space brake; the
pad's right and left triggers and the left stick, A throttle, X brake; on
a phone the HUD's arrows and Brake) through the banked sweeper, down the
back straight and over the jump (it needs about 75 km/h at the lip, or the
car drops into the gap and is put back past it: that lap no longer
counts), round the banked hairpin and through the S to the line. Every lap
has two splits, each shown against the record's; a lap that beats the
record becomes it, and from then on its ghost drives every lap beside
you. R (the pad's Y) puts the car back on the road. P or the HUD's button
pauses (Resume, Restart, the ghost on or off, Settings, Quit). The results
give the race's time against the session's best, the laps, the stars
(laps under 40, 37 and 35 s) and the rank on the leaderboard of best laps;
Watch replay drives the whole race again from the countdown.

## Run it

```sh
bun tools/make-assets.ts        # once after a clone: every asset, the 11 MB music and the style's art and fonts (not committed)
og cook .
og run . --headless --commands "1:screenshot:title.png;1.2:race;4:drive:auto;20:screenshot:sweeper.png;20.1:quit"
og test .                       # the ten playtests, the reference frames against references/<tier>/
OG_FX_CAPTURE=1 og test .       # records this tier's reference frames
og check .                      # tsrs over src, tests and tools
```

Inside the engine repository `og` is `target/debug/og`, and the playtests
load the dev addon at `packages/engine/native/`. `bun tools/make-assets.ts
--no-music` remakes everything but the music. Every asset is made by code,
the same bytes on every run: the circuit (road, walls, embankments, neon
tubes, gantry and gates, skyline and ridges) from the centre line in
`src/track.ts`, the ground grid, the car, its ghost and the wheels, the
mini-map, the smoke and skid textures, the emitters, the look, every sound
and the music. The UI style is the engine's neon pack, copied from
`templates/ui-kit` by the same script and retinted.

## What it exercises

| Area | How | Where |
|---|---|---|
| A generated circuit | a closed centripetal Catmull-Rom spline through 22 control points, resampled every 2 m (478 samples, 956 m); a 14 m road banked into each bend (the outside edge raised up to 25°, the bank smoothed over 12 m, the low edge kept on the ground); a kicker ramp to a 3.4 m lip, a 15 m gap, a landing ramp; walls along both edges; one triangle-mesh collider for the road (it grips) and one for the walls (slick) | `src/track.ts`, `tools/circuit.ts`, `src/world.ts` |
| `og.Vehicle` | a raycast car on a 1,000 kg dynamic chassis (rear drive, 5.2 kN a wheel, 46 m/s), its tuning in a `Tuning` resource; a flat chassis box with the wheel mounts above its centre, so the car corners without rolling; the wheel eased to the asked lock and the lock shortened with speed; roll and pitch damped, a little downforce and, in the air, the nose turned to where the car flies (`airAlign`, so a twist off the kicker's lip does not land it sideways), all through `og.Velocity` (FINDINGS #2); the wheel models placed from `og.physics.vehicleWheels` | `src/car.ts`, `src/world.ts`, `src/fx.ts` |
| Lap timing | progress along the centre line (the nearest sample, searched near the last); two splits and the line, forward only and in order; laps, sectors and the record in ticks; a delta against the record at each split; resets (R, upside down, or a second in the gap) put the car back and the lap no longer counts | `src/race.ts` |
| The ghost | the car's pose every second tick of a lap (15 characters a pose, `src/tape.ts`) on the current lap's tape; a lap that beats the record keeps its tape as the best, stored with the record; the ghost (a translucent model and wheels, no body) drawn from the start of every lap where the record's car was, to the centimetre (FINDINGS #3 says why poses and not a second car replaying the inputs) | `src/ghost.ts`, `src/tape.ts`, `tests/ghost.og.ts` |
| The replay | the world saved in the countdown (`world.snapshot()` through `og.platform.saveSlot`) and the run's inputs taped two characters a tick; Watch replay restores the world (`world.restore`) and feeds the inputs back: the same race, every lap on its tick | `src/replay.ts`, `tests/rerun.og.ts` |
| og.platform | the record and its ghost in the store at the revision read (a conflict read again, the faster kept); the settings (with the language and the ghost switch); the run's best lap on the `laps` board (seconds, ascending), the top ten and the ten around the player | `src/save.ts`, `tests/save.og.ts`, `tests/lap.og.ts` |
| Engine audio | a synthesised four-cylinder loop pitched by the revs (rising through each of five gears, dropping at the shift) and loudened by the throttle; a tyre squeal faded in with the slide; knocks, landings, the countdown's beeps, GO, split and lap chimes, the record's fanfare; "Meridian" (57.6 s of synthwave, streamed in 4-second chunks) on the music bus ducked under the effects; a limiter on the master | `tools/audio.ts`, `src/sound.ts`, `tests/audio.og.ts` |
| Effects | tyre smoke from the rear wheels while sliding (`og.ParticleSystem` start and stop ticks); skid marks, a ring of 28 `og.Decal` streaks laid every 1.1 m of a slide and fading; sparks and a shake on a wall; dust, a shake and a punch on a landing | `src/fx.ts`, `assets/fx/` |
| A look | `looks/meridian.ogt` extends `neon-vector` (AgX, bloom, a violet fog, a dusk sky with stars, a vignette; the preset's hull outline off, FINDINGS #7) | `tools/make-assets.ts` |
| og.ui on the neon pack | `mainMenu`, `racingHud` as a time trial (speed, gear, lap, the lap time with the last split's delta under it, the record, the three sectors as its splits, no place and no boost meter, the mini-map with the car and the ghost, touch steering; every word of it from the tables), the countdown, `pauseMenu`, `settingsScreen` (every word from the tables), `resultsScreen` (the race's time as its headline, counted up as m:ss.mmm, against the session's best race), `leaderboardScreen` (honest loading, error and empty states); a wipe between screens; toasts | `src/ui.ts` |
| Two languages | `strings/en.ogt` and `strings/de.ogt` (German: the longest labels, umlauts in the pack's Latin-1 faces) | `assets/strings/`, `tests/locales.og.ts` |

## Playtests

| File | Proves |
|---|---|
| `drive.og.ts` | held on the line in the countdown; W answered the next tick; speed from a standing start; the brake; a slide with smoke and a knock on the wall; the jump at speed with a landing; a fall into the gap put back past it, the lap not counted; R; the pad's trigger and stick |
| `lap.og.ts` | three autopilot laps: splits in order, laps summing to the race, the record and its sectors, the record and its poses stored, the best lap on the board, the results' times, the leaderboard's row |
| `ghost.og.ts` | the record's ghost on the second lap where the car was on the first, to the centimetre, at three times into the lap; the ghost switched off |
| `rerun.og.ts` | Watch replay: the countdown's world restored and the taped inputs driving the same three laps, tick for tick; nothing new on the board |
| `save.og.ts` | a relaunch (`store: "keep"`): the record and its ghost back, the title in German, the stored ghost driving the new race's first lap where the first process's car drove |
| `audio.og.ts` | the engine's pitch from the grid to speed, the beeps and GO, the tyres in a slide and after it, the music streamed and looping, a mix that is heard and never clips |
| `locales.og.ts` | the title, the leaderboard, the HUD, pause, the settings and the results in German, no English of the game's table left, the racing HUD's own words included (`Runde` under the lap, no `lap`, no boost meter; FINDINGS #8 fixed and taken up); no control unnamed |
| `replay.og.ts` | a session of slides, the jump, resets, pause and the ghost run twice in fresh processes ends on one world hash |
| `frames.og.ts` | ten reference frames at 1280 x 720 (`references/metal/`): the title, the jump, a slide, the countdown, the sweeper, the ghost, pause, the results, the leaderboard, the German settings |
| `published.og.ts` | input and screenshots only (Time trial pressed, the countdown, W held up the start straight, Escape): four frames (`pub-*.png`) the published page is held to as well (`packages/engine/tests/web/published.ts --playtest tests/published.og.ts`) |

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `race[:go]` | a race from the grid now, as Time trial does; `go` skips the countdown (and with it the replay's saved start) |
| `drive:<auto\|off>` | the autopilot (pure pursuit on the centre line, its speed from the bends ahead) drives the car, or stops |
| `warp:<k>[:<speed>]` | the car k samples (2 m each) past the start line, along the road, at that speed in m/s |
| `reset` | the car back on the road, as R |
| `language:<en\|de>`, `open:<settings\|board\|pause>`, `ghost:<on\|off>` | the language, a screen, the ghost switch |

`get:Race` answers the whole run (phase, lap, clock, splits, laps,
record and its sectors, the ghost, the replay, the counters the playtests
read); `facts` give the same with the car's position.

## Layout

`src/track.ts` the circuit's numbers (shared with `tools/`);
`src/components.ts` the state; `src/world.ts` the circuit, the cars and the
effects' entities; `src/race.ts` the countdown, the laps and the resets;
`src/car.ts` the driver and the chassis; `src/ghost.ts` the tapes and the
ghost; `src/tape.ts` and `src/tapes.ts` the tapes' encoding and chunks;
`src/replay.ts` the replay; `src/fx.ts` the wheels, smoke, sparks, dust and
skid marks; `src/view.ts` the camera; `src/sound.ts` the voices;
`src/ui.ts` the screens; `src/save.ts` the platform; `src/main.ts` the game,
its commands and facts. Systems in tick order: Ui, Race, Driver, Tapes,
Chassis, Fx, Marks, View, Voices.

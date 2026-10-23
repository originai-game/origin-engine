# Ember Vale — the third-person archetype (3D)

Dusk is falling on a lake vale and four beacons stand dark on its knolls.
The lamplighter walks, runs, wades and jumps there; holding F by a beacon
it walks up to the plinth, reaches into the bowl and, a moment later, the
beacon catches. Three stone sentinels walk their rounds of the lake path:
one that sees the lamplighter calls the alarm and runs it down, and its
grab costs a heart. The lake is a refuge (the sentinels' navmesh stops at
the shore) but wading is slow. Every beacon lit before night wins the run;
three hearts gone, or night, loses it. Then the summary, and again.

余烬山谷：暮色正落在湖谷上，四座烽火台在小丘上黑着。点灯人可以走、跑、涉水、跳跃；在烽火台旁按住 F，它会走到台前，把手伸进火盆，片刻之后烽火燃起。三个石头哨兵沿湖边小路巡逻：看见点灯人的哨兵会发出警报并追上来，被抓住就失去一颗心。湖水是避难所（哨兵的导航网格止于岸边），但涉水很慢。入夜前点燃全部烽火即获胜；三颗心都失去或夜幕降临则失败。随后是总结，再来一次。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It is the engine's mid-size 3D slice: one
third-person loop that uses the 3D systems together rather than a station
per feature (templates/fx-showcase has the stations).

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.6:run;0.8:warp:0;1:key_down:KeyF;3:screenshot:beacon.png;3.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships (behind "tap to play")
```

## What it demonstrates, and the skills it exercises

- **Terrain and water** (og-terrain, og-render §water): `assets/terrain/vale.ogt`
  is a 128 m heightmap (a meadow, a lake basin, a knoll under each beacon,
  a ring of cliffs) splatted from a control map with grass, path, rock and
  sand (rock and sand by slope and height rules), with its heightfield
  collider. `og.Water` fills the lake at y 1: waves, planar reflection,
  shadows, caustics, a depth fade and foam where it meets the shore and
  the lamplighter's legs.
- **A character on og.CharacterController** (og-physics): a capsule with a
  slope limit, step height, ground snap and coyote time; the controls are
  relative to the view (WASD or the arrows, the left stick, the touch
  joystick); Shift or a full stick runs; Space, A or Jump jumps; wading
  slows to `Tuning.wade`.
- **Skeletal animation with blend trees and IK** (og-anim): the hero is
  the strider, its clips retargeted from the humanoid clip library at cook
  time (`assets/models/strider.glb.ogt`); the sentinels are the stout, the
  same library retargeted at run time (`og.anim.retarget`). Each plays an
  `og.AnimationGraph` (`src/anim.ts`): a 1D blend of idle, walk and run by
  speed, and for the hero a cross-fade to an air state. Kindling raises an
  `og.IkChain` (FABRIK, three bones) on the right arm into the bowl; the
  playtest reads it reached within 2 cm.
- **NPCs on a navmesh with behaviour trees** (og-nav, og-ai): `assets/nav/vale.ogt`
  bakes the terrain above the waterline and the plinths at cook time; each
  sentinel is an `og.NavAgent` thinking with `assets/ai/sentinel-<name>.ogt`
  (patrol its round, alert and chase on sight, grab within reach, search
  where it last saw the hero) and seeing through `og.Perception` (a 120°
  cone, its sight from `Tuning`, the terrain and the plinths occlude).
- **Particles, decals and the post stack** (og-render): each beacon's fire,
  smoke and a one-shot burst of embers (`assets/fx/*.ogt`), a splash at
  every wading step and landing in the lake; a scorch decal on the knoll
  and a rune on the plinth that glows once lit; bloom, AO, god rays, a
  vignette, the teal-orange LUT and film grain, the sky, height fog and
  `og.DayNight` running from late afternoon to night as the run's clock.
- **Screens on og.ui with tokens and MSDF text** (og-ui §2–§6, og-taste):
  the engine's fantasy pack as `assets/styles/ember` (Cinzel and Newsreader,
  Noto Sans SC as the fallback, every font an MSDF atlas at cook time); the
  title over an orbit of the vale, the HUD (hearts and beacons top-left,
  the dusk clock and Pause top-right, the kindling prompt and its progress
  at the bottom; on a touch screen a joystick, Jump and Kindle), toasts,
  `og.ui.pauseMenu`, a settings dialog and the summary built from widgets,
  a dissolve between screens (a fade with reduced motion). Spacing comes
  from the tokens' scale; colours are token references.
- **Feel** (og-feel): a camera punch when a beacon catches; a grab is a hit
  stop, a red flash, a shake and a knock-back; a sentinel's eye burns red
  while it chases, with "!" over it when it sees you.
- **Sound** (og-audio): a voice per event (pitch-varied steps and splashes,
  the jump, the ignition, the alarm, the grab, the dawn and night
  stingers), the UI's sounds from `og sfx` presets, the vale's loop on the
  music bus ducked under the effects and a little reverb on the effects.
- **Tune**: every `Tuning` field is tunable (speeds, wading, kindling,
  dusk, the sentinels' pace and sight, the hit stop, the camera).
- **Languages** (og-i18n): every string is a key of `assets/strings/en.ogt`
  and `zh-CN.ogt`; the Chinese font is Noto Sans SC subset to exactly the
  table's characters.
- **The platform** (og-platform-sdk): settings through
  `loadSettings`/`saveSettings`; the record (`{runs, wins, best}`) read on
  the first ticks and written at the revision read; one call at a time
  (a newer record waits for the read or write in flight and goes out at
  the revision its answer gives); a write refused as stale (another
  device played meanwhile) is read again, merged (the stored counts plus
  what this device added, the better best time) and written at the new
  revision; a `run.over` telemetry event.
- **The tap gate**: `og build . --publish` ships behind the default
  `gate: "tap"`, so audio starts on the player's first tap.

## Rules

- **Moving.** Walk 3 m/s, run 6 m/s, wade at 0.45 of either. The view
  turns with Q and E (the right stick, or on a touch screen a finger
  dragged sideways across the right half, between the clock and Jump)
  and swings slowly behind a walking hero.
- **Kindling.** Within reach of a dark beacon the prompt shows; holding F
  (X, Kindle) walks the hero to the plinth and reaches into the bowl;
  1.6 s of reaching lights it. Letting go lets the kindling cool at twice
  the rate.
- **Sentinels.** Each patrols its round at 1.6 m/s, sees 13 m in a 120°
  cone (not through the terrain or the plinths), chases at 4.2 m/s and
  grabs within 1.5 m: a heart, a knock-back and 1.6 s spared. It cannot
  follow into the lake; out of sight it searches for 4 s, then goes back.
  A new run puts each back at the start of its round, on patrol.
- **The end.** All four lit: dawn will find the vale lit (a win; the best
  time is kept). The hearts gone: caught. 150 s of play (the hour reaching
  19:30): night. 1.5 s later the summary; Again starts a run from nothing,
  Title goes back. Only the record carries over.

Everything is deterministic: the clock is the fixed tick, og.ai, og.nav
and the character controller are deterministic and in snapshots, and
`tests/replay.og.ts` replays a run (moving, a jump, kindling, a grab) to
the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| walking, running | the blend from idle through walk to run by speed, a footstep a stride (pitch-varied) | `Hero` in `src/hero.ts`, `src/anim.ts` |
| wading | slow, a splash at each stride, foam at the legs | `Hero` |
| a jump | the air state cross-fades in, a jump sound; a landing in the lake splashes | `Hero` |
| kindling | the hero walks up and the hand reaches into the bowl (IK), the progress bar fills | `Hero`, `Beacons`, `Ui` |
| a beacon catches | fire, smoke and a burst of embers, its light swelling in over 0.5 s and flickering, the scorch and the glowing rune, an ignition, a camera punch, a toast (not for the last) | `catchBeacon` in `src/beacons.ts`, `lightBeacon` in `src/world.ts` |
| seen | "!" over the sentinel, an alarm, its eye burning red, it runs | `alarm` in `src/sentinels.ts` |
| grabbed | a 5-tick hit stop, a red flash, a shake, the hero thrown back, a heart gone | `grab` in `src/sentinels.ts` |
| dusk | the sun lowers and the light warms, then cools, the clock's bar empties | `clock` in `src/ui.ts` |
| the end | the dawn or night stinger, the summary after a beat with its icon popping in | `clock`, `summary` in `src/ui.ts` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase and ending, the record and its requests, hearts, lit beacons, the run's time and hour, kindling, wading, the counts (jumps, splashes, grabs, alarms), the knock-back, the view's yaw, the HUD's stick and buttons, sound requests, the terrain and navmesh handles |
| `Tuning` | resource | the knobs below |
| `Prefs` | resource | the volumes, the language, reduced motion |
| `Hero` | the hero's capsule | steps, air time |
| `Beacon` | the four plinths | index, lit, when it caught, the bowl's rim |
| `Sentinel` | the three sentinels | index, grabs, alarms, speed, alert |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `og.systems.cameras` | the camera's punch and shake |
| `Ui` | the screens, starting and ending runs, the clock and the hour, pausing, the buses, the platform's answers |
| `Hero` | the controls, the character controller, the graph's parameters, the reach (IK), steps, splashes and jumps |
| `View` | the title's orbit, the follow camera kept above the ground and the water |
| `Sentinels` | their sight and pace from `Tuning`, the graph's speed, the feet on the terrain, the eye's light |
| `Beacons` | which beacon is in reach, kindling, catching, the lights' swell and flicker |
| `Voices` | plays each requested sound once |

og.nav, og.ai (the sentinels' decisions, whose commands `alarm` and `grab`
run before the next tick), physics and animation run in the engine's own
stages.

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `walkSpeed`, `runSpeed` | 3, 6 m/s | the hero |
| `wade` | 0.45 | the speed in the lake, as a fraction |
| `jumpSpeed` | 5.2 m/s | the jump |
| `kindleSeconds`, `reach` | 1.6 s, 2.4 m | how long a beacon takes, how near the hero must be |
| `duskSeconds` | 150 s | the run's length before night |
| `patrolSpeed`, `chaseSpeed`, `sight` | 1.6, 4.2 m/s, 13 m | the sentinels |
| `hitStop` | 5 ticks | a grab's hit stop |
| `cameraDistance` | 6.2 m | the follow camera |

## What a Maker changes first

1. **The vale**: `tools/make-assets.py` (the knolls, the lake, the paths,
   the sentinels' rounds), then `python3 tools/make-assets.py && og cook .`;
   keep `BEACON_AT`, `LAKE` and `START` in `src/world.ts` and the plinths in
   `assets/nav/vale.ogt` in step.
2. **The pressure**: `duskSeconds`, the sentinels' speeds and sight, their
   minds in `assets/ai/`, `HEARTS` in `src/components.ts`.
3. **The look**: the post stack, fog and sky in `buildVale`, the water's
   colours, the skin's palette in `assets/styles/ember/style.ogt`.
4. **The characters**: another CC0 humanoid in `assets/models/` with the
   clip library retargeted onto it (`src/anim.ts`), the graphs' thresholds.
5. **Words**: `assets/strings/*.ogt`; a new Chinese string needs the script
   run again to subset the font (with fontTools and the project template's
   Noto Sans SC; without them the script fails naming the characters the
   committed font lacks).

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `run` | starts a run now, as Set out does |
| `warp:<beacon>` / `warp:<x>:<z>` | puts the hero where it kindles a beacon (0 to 3), or on the ground at x, z |
| `light:<beacon>` | the beacon catches now (the fourth wins) |
| `hearts:<n>` | sets the hearts left (0: caught) |
| `dusk:<seconds>` | jumps the run's clock (night at `Tuning.duskSeconds`) |
| `stale_record` | another device writes the record, so this one's next write is refused as stale and merged |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `alarm:<sentinel>`, `grab:<sentinel>` | the sentinels' behaviour commands: sentinel 0 to 2 (west, north, east) calls the alarm or grabs (og.ai sends them with the sentinel's entity) |
| core actions | `touch_move_to:play`, `Pause`, `Jump`, `Kindle`, `Stick`, `Again`, `Menu`; WASD, the arrows, Shift, Space, F, Q, E, Escape; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the ending, hearts, lit
beacons, the time and hour, the near beacon and kindling, wading,
grounded, speed, jumps, splashes, grabs, alarms, the record and the
language. The playtests read the same values from the `Game` resource with
`get:Game`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Set out by tap; walking 3 m/s and running 6 m/s north to the lake, wading at 2.7 m/s with splashes; at a beacon the prompt, F held: the IK chain reaches the bowl (error ≤ 2 cm), the beacon catches after kindleSeconds, the HUD and toast count it |
| `tests/sentinels.og.ts` | the west sentinel walks its round calm; the hero in its view: the alarm, its eye's light, a chase and a grab that costs a heart |
| `tests/win.og.ts` | four beacons lit: dawn, the record merged after a stale write (runs, wins, the best time) with a `run.over` event, the summary, then Again from nothing |
| `tests/lose.og.ts` | caught with no hearts left, Again, night at dusk's end, Title |
| `tests/hitstop.og.ts` | Escape pressed during a grab's hit-stop still pauses |
| `tests/restart.og.ts` | a sentinel chasing when the run restarts is back on patrol, its eye calm |
| `tests/pause.og.ts` | Escape holds the run (time scale 0, the clock stopped, keys move nothing); settings switch to 简体中文 and move the music; Done saves through og.platform; Resume |
| `tests/phone.og.ts` | 1170 × 2532 (390 × 844 at 3×): the HUD inside the safe area, touch-sized controls, the joystick walks, Jump jumps, Kindle held lights a beacon |
| `tests/frames.og.ts` | the title, a beacon catching and the dawn summary match `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a run with movement, a jump, kindling and a grab, run twice, ends on one hash |

## Assets

Nothing here is downloaded. `tools/make-assets.py` regenerates the vale's
heightmap and control map, the sounds and music, the `og sfx` presets, the
sentinels' minds and the font subset byte for byte; the characters, the
clip library, the textures and the beacon's effects are the engine's own
CC0 assets. `LICENSES.md` lists every file; the skin's are in
`assets/styles/ember/LICENSES.md`.

## Known limits (engine gaps, not design)

- `og.IkChain`'s `space: "world"` is the frame of the skeleton entity's
  parent (its og.Transform applies, the parent's does not), so a target
  for a body parented to the capsule is relative to the capsule
  (`src/hero.ts`); the API's doc says only "the skeleton entity's
  og.Transform applies".
- `og.DayNight` advances on unscaled ticks, so a pause would not hold it;
  the game holds it (speed 0) and sets the hour itself (`setHour` in
  `src/world.ts`).
- og.ui's `settingsScreen` and `resultsScreen` carried English labels of
  their own when this game was written, so the settings dialog and the
  summary are built from widgets (both now take a field for each of their
  words: og-i18n §2).

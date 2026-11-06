# Holdout — the shooter archetype (3D)

Dusk over an outpost's landing field, ringed by canyon walls with three
gates. Five waves come through them: walkers that close in, wind up and
strike, and drones that hover, find a clear line and charge a bolt down
it. The trooper holds the field with a hitscan pulse rifle (recoil, bloom,
a muzzle flash, tracers, sparks and bullet holes, coolant sprays from the
foes it hits), 24 rounds a magazine and a reserve to reload from, behind
the cargo stacks, the drill and the low walls. Clear wave 5 and the line
held; every trooper down and it is overrun. Two seats hold it together, or
duel each other to five frags.

坚守：黄昏时分，前哨站的着陆场被峡谷岩壁环绕，开有三道闸门。五波敌人从闸门涌入：步行机会逼近、蓄力、重击；无人机悬停，找到无遮挡的射线后蓄能射出电弧。士兵手持即时命中的脉冲步枪（后坐力、散布扩大、枪口火焰、曳光、火花与弹孔，被击中的敌人喷出冷却液），弹匣 24 发，用备弹换弹，借货箱、钻机和矮墙作掩护。清完第五波即守住阵线；所有士兵倒下则被攻破。两个座位可以并肩坚守，也可以对决，先拿下五次击倒者胜。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. The `shooter` starter (`packages/create/bin/starters.ts`)
starts from it in all three of its views, and so do the `arena` genre's 3D
views.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.5:run;0.6:warp:-3:2;0.7:aim:0:-5;0.8:spawn:walker:-3:-8;1:key_down:KeyF;2:screenshot:fight.png;2.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships (behind "tap to play")
```

## What it demonstrates, and the skills it exercises

- **Three views of one game** (`src/view.ts`, the same file in every 3D
  starter, and `src/hero.ts`): `src/creation.ts`'s `view` picks
  - **first person** (the default): `og.FirstPersonCamera` at the eye, the
    rifle in the hands on a layer only that seat's camera draws, the
    trooper's own body left out of its own view (its shadow still falls);
  - **third person over the shoulder**: `og.FollowCamera` behind the look,
    offset 0.62 m right by the `Shoulder` system, pulled in short of any
    wall between the head and the camera (a raycast); the rounds leave the
    trooper's head along the camera's line, so the crosshair is honest;
  - **isometric twin-stick**: the camera at the look's fixed angle; the
    move is relative to the screen, the look (the mouse, the right stick,
    a drag) aims and turns the trooper, the rifle fires level from the hip
    and the foes are hit on the floor's plane; the canyon ring and the
    skyline on the camera's side are not drawn (`cutAway` in
    `src/world.ts`; their colliders stay), so the field is never behind a
    wall.
- **A hitscan weapon** (`src/weapon.ts`): each round a ray through
  `og.physics.raycast` for the arena (and a duel's other trooper) and the
  foes' own shapes (a capsule body and a head sphere: a headshot doubles
  the damage), inside a spread cone (`og.Rng`, so a room fires the same
  rounds) that blooms as the rifle fires and settles; recoil climbs the
  view and drifts it; an `og.Beam` tracer from the muzzle, a flash mesh
  and its light, sparks (`assets/fx/sparks.ogt`) and a bullet hole
  (`og.Decal`, the oldest of 24 reused) where a round meets the arena, a
  coolant spray where it meets a foe; a camera kick and shake for the one
  who fired.
- **Foes on a navmesh** (og-nav, `src/foes.ts`): `assets/nav/arena.ogt`
  bakes the floor around the cover at cook time; walkers (the stout,
  tinted, with a glowing visor, its clips from the humanoid library
  retargeted at run time) and drones (`assets/models/drone.glb`) are
  pooled `og.NavAgent`s on an `og.Crowd`, drawn through a gate by a wave
  and put back when downed. A walker hunts the nearest standing trooper,
  winds up 0.5 s within 1.75 m and strikes within 2.25 m; a drone hovers
  1.9 m up, closes in until it has a clear line within 13 m, then charges
  0.95 s with an aiming beam that follows for 0.6 s and locks (the beat to
  dodge) before the bolt.
- **Feedback** (og-feel): a hit flash and the kit's hit marker on every
  hit (red for a kill), a headshot's marker, a fall with a burst and a
  3-tick hit stop on a kill, the damage arc toward whatever hurt you, the
  low-health edge, an empty click, the reload's dip of the rifle.
- **The shooter kit's HUD** (`@origingame/engine/extensions/shooter`): the
  crosshair (its spread follows the bloom) with hit markers, the ammo
  counter (magazine pips, the reserve, the reload ring and its hint), the
  killfeed, the damage direction and the low-health vignette; the wave,
  the foes left and the score at the top; health bottom left and the
  rounds bottom right (on a touch screen both under the top line, leaving
  the bottom to the controls; two seats on a phone get the rounds alone,
  without the name and pips, to fit their halves); Pause top right.
- **Screens on og.ui and the scifi style** (`assets/styles/scifi`): the
  title and the pause are the game's screen documents
  (`assets/ui/main-menu.ogt`, `pause.ogt`); the settings (the volumes, the
  look speed, the language, reduced motion) and the summary are built from
  widgets; a dissolve between screens (a fade with reduced motion).
- **The arena** (`tools/make-assets.py` writes `assets/scenes/arena.ogt`):
  a 36 m field ringed by 6 m canyon blocks with gates north, west and east;
  the cover is KayKit's Space Base Bits (CC0), each piece with a box
  collider; lamp posts at the gates; the outpost's skyline beyond the ring;
  dusk light, a sun with shadows and thin height fog (`buildArena` in
  `src/world.ts`), the faceted-low-poly look.
- **Sound** (og-audio): a voice per event (the shot, the reload, the empty
  click, a hit, a fall, a strike, the drone's charge and bolt, the wave's
  horn and its clear, the win and loss stingers, steps and the jump), the
  UI's sounds from `og sfx` presets, the arena's loop on the music bus.
- **Languages** (og-i18n): every string is a key of `assets/strings/en.ogt`
  and `zh-CN.ogt`.
- **The platform** (og-platform-sdk): settings through
  `loadSettings`/`saveSettings`; the record (`{runs, wins, best}`) read on
  the first ticks and written at the revision read, a stale write read
  again and merged; a `run.over` telemetry event. A room keeps no record.

## Seats: alone, two on one device, two over a room

Every trooper is a seat (`src/seats.ts`, the same file in every starter;
`src/creation.ts` holds the creation choices and the controls): the game
reads `og.PlayerInput` alone, so Holdout plays

- **alone**: seat 0 from the keys (both sets), any pad, the mouse while
  the pointer is locked, and on a touch screen the game's own HUD;
- **local-multiplayer**: a second trooper (tinted blue) beside the first,
  the screen split in halves (an `og.CameraView` per seat), each half with
  its own crosshair, health and rounds; seat 0 on WASD, F, Space, R (pad
  1), seat 1 on the arrows, `/`, Enter, `.` (pad 2);
- **online-lockstep**: two peers over a room, each looking out of its own
  seat (`og.Lockstep.localPlayer`), every peer running the whole arena.

**The design with two seats.** The holdout is co-op: the waves are half as
big again (×1.5), each foe hunts the nearest standing trooper, the score is
shared, a downed trooper is back on its feet (at 60 % health) when the wave
is cleared, and the run is overrun only when both are down. Rounds pass
through a partner. The title also offers **Duel**: the same arena with no
foes, first to five frags; a downed duellist is back after 2.5 s at the
duel point farthest from the other. Menus act through the seats: Hold the
line and Again press Jump, Duel presses Reload, Title and Leave the arena
press Reload, Pause and Resume press Pause, so a room acts on one tick on
every peer. `bot:<n>` drives this device's seats from a script (the room
test).

## Rules

- **Moving.** 5.2 m/s on `og.CharacterController`, a 5.4 m/s jump; the
  move is relative to the view (first and third person) or the screen
  (isometric).
- **The rifle.** 8 rounds a second held, 17 damage a round (34 a
  headshot), 24 a magazine, 120 in reserve, a 1.35 s reload (R, X, Reload);
  an empty magazine fired clicks and reloads itself. 0.5° of spread at
  rest, 0.35° more a round, settling back once it stops.
- **Waves.** Five: 5, 7, 8, 10 and 13 foes (drones from wave 3), a 4 s
  breather before each (2.5 s before the first). Clearing a wave scores
  250 times its number, refills the reserve, gives back 35 % health and stands a downed
  partner up.
- **Foes.** A walker has 70 health and strikes for 14; a drone 45 health
  and its bolt 11. Each wave the walkers are 0.2 m/s faster. A walker
  scores 100, a drone 150, a headshot kill 50 more.
- **The end.** Wave 5 cleared: the line held (a win; the best score is
  kept). Every trooper down: overrun. A duel ends at five frags. 1.5 s
  later the summary (wave, score, foes down, accuracy, time, best); Again
  starts a run from nothing, Title goes back.

Everything is deterministic: the fixed tick, `og.Rng` for the spread and
the recoil, og.nav and the character controller in snapshots;
`tests/replay.og.ts` replays a run (moving, a jump, firing, a kill, a
drone's bolt) to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| firing | the flash and its light, a tracer, the view climbing and drifting, the crosshair opening, a kick and a touch of shake | `shoot` in `src/weapon.ts` |
| a hit | sparks and a bullet hole on the arena; on a foe its flash, a coolant spray and the hit marker | `shoot`, `hurtFoe` in `src/foes.ts` |
| a kill | the fall and sink, a burst, a 3-tick hit stop, the red marker, the killfeed | `hurtFoe`, `killfeed` in `src/ui.ts` |
| a walker's strike | the windup (its visor and eye flaring), the swipe, the damage arc and a red flash | `walker` in `src/foes.ts` |
| a drone's charge | the beam warming and locking on, its eye burning, then the bolt | `drone`, `fire` in `src/foes.ts` |
| a reload | the rifle dips, the ammo counter's ring fills | `pose` in `src/hero.ts`, `ammo` in `src/ui.ts` |
| low health | the kit's low-health edge pulsing | `seatHud` in `src/ui.ts` |
| a wave | the horn and "Wave n incoming" with its count; the clear's sting | `Waves` in `src/foes.ts`, `topLine` in `src/ui.ts` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, mode and ending, the waves, the score and counts, the record and its requests, sound requests, the effects' next slots, the troopers' and navmesh's handles |
| `Tuning` | resource | the knobs below |
| `Hero` | each trooper's capsule | its seat, body, rifles and arms, last tick's buttons, health, rounds, the reload and cooldown, recoil and bloom, the last hit and hurt, kills, frags, score, facing, steps and jumps |
| `SeatLook` | each trooper (`src/view.ts`) | its view's yaw and pitch, turned by its seat's look |
| `Foe` | the pooled walkers and drones | kind, state, health, timer, the seat it hunts, a drone's locked aim, its flash, body, beam and eye |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `Seats` | each seat this device plays: its keys, pad, mouse-look and touch HUD to `og.PlayerInput` |
| `ViewRig` | each trooper's look from its input, the cameras (and with two seats here each half's view) |
| `Shoulder` | third person: the camera over the right shoulder, in short of walls |
| `Ui` | the screens, the seats' choices (start, duel, pause, leave), ending runs, the buses, the platform's answers, the HUD |
| `Hero` | each trooper from its input: the character controller, jumps and steps, the body's pose, the rifle at the shoulder, the arms' IK, the first-person hands |
| `Weapon` | the reload, the rounds, what they hit, the recoil and its settle, the tracers and the flash |
| `Foes` | the walkers' and drones' hunts, windups, strikes and bolts, their falls |
| `Waves` | the breathers, the gates' spawns, a wave's clear, the last wave held |
| `og.systems.cameras` | the camera's kick and shake |
| `Voices` | plays each requested sound once |

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `moveSpeed`, `jumpSpeed` | 5.2, 5.4 m/s | the trooper |
| `fireRate`, `damage` | 8 a second, 17 | the rifle (a headshot doubles the damage) |
| `magazine`, `reserve`, `reloadSeconds` | 24, 120, 1.35 s | the rounds |
| `recoil`, `spread`, `bloom` | 0.6°, 0.5°, 0.35° | the kick a round, the cone at rest, what each round adds |
| `health` | 100 | a trooper |
| `walkerHealth`, `walkerSpeed`, `walkerDamage` | 70, 2.8 m/s, 14 | the walkers |
| `droneHealth`, `droneDamage`, `droneRange` | 45, 11, 13 m | the drones |
| `breather` | 4 s | between waves |
| `hitStop` | 3 ticks | a kill's hit stop |

## What a Maker changes first

1. **The arena**: `tools/make-assets.py` (`COVER`, `LAMPS`, `SKYLINE`,
   the gates, the spawns and duel points), then `python3 tools/make-assets.py && og cook .`;
   the navmesh bakes from the scene.
2. **The waves**: `WAVE_FOES` in `src/foes.ts` (a letter a foe, `W` a
   walker, `D` a drone), `WAVES` and the pools in `src/components.ts`, the
   foes' tunables.
3. **The rifle**: the rate, damage, magazine, recoil and spread in
   `Tuning`; `RANGE` and the settle in `src/weapon.ts`; its model is
   `blaster()` in `tools/make-assets.py`.
4. **The look**: `assets/looks/holdout.ogt` (another catalogue style),
   the light and fog in `buildArena` (`src/world.ts`), the style's palette
   in `assets/styles/scifi/style.ogt`.
5. **The title and the pause**: `assets/ui/main-menu.ogt` and
   `pause.ogt`: a button is an element with `on click emit <event>`,
   which `src/ui.ts` answers; its words are string keys.
6. **Words**: `assets/strings/*.ogt`.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `run`, `duel` | starts a holdout (as Hold the line does), or a duel (two seats) |
| `warp:<x>:<z>[:<seat>]` | puts a trooper on the floor at x, z |
| `aim:<yaw°>:<pitch°>[:<seat>]` | turns a trooper's look (yaw 0 north, growing to the left) |
| `wave:<n>` | the breather before wave n, the standing foes gone |
| `clear` | downs every standing foe and drops the rest of the wave |
| `spawn:<walker\|drone>:<x>:<z>` | a foe from its pool, hunting from there |
| `hurt:<amount>[:<seat>]` | a trooper takes damage from straight ahead |
| `ammo:<magazine>[:<reserve>]` | seat 0's rounds |
| `bot:<n>` | a scripted player drives this device's seats (0 hands back) |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `stale_record` | another device writes the record, so this one's next write is refused as stale and merged |
| core actions | `touch_move_to:Play`, `Duel`, `Pause`, `TouchMove0/Stick`, `Fire`, `Jump`, `Reload`, `Again`, `Menu`; WASD, F, R, Space, Escape; `gamepad:…`, `mouse_move` with `pointer_lock:on` |

`facts` (the probe's `state()`) reports the phase, mode, ending and
winner, paused, the wave and foes standing, the score and counts, the
record, the language and each seat (`seat0`, `seat1`: health, down,
rounds, reloading, kills, frags, where it stands and looks). The playtests
read the same values from `Game` and the troopers' rows with `get:`.

## Playtests (`og test .`)

Every playtest reads the seats, the view and the players from
`src/creation.ts` (`tests/lib.ts`), so the same files pass in every
scaffold the starter makes.

| File | Proves |
|---|---|
| `tests/seats.og.ts` | W, the first pad's stick, its right stick, the mouse (pointer locked), F and the touch stick each reach seat 0's `og.PlayerInput`, and the trooper walks (north, or up the screen isometric), turns and fires from it |
| `tests/flow.og.ts` | Hold the line by tap; wave 1 after the breather, walkers pathing in; a walker shot down (hits, the kill and its points on the trooper and the run, each round from the magazine); R reloads from the reserve; an empty magazine reloads itself |
| `tests/win.og.ts` | a cleared wave: the breather, the reserve refilled, a downed partner back (two seats); wave 5 cleared holds the arena; the summary's win and best; Again |
| `tests/lose.og.ts` | a walker's strikes hurt; every trooper down is an overrun; the summary; Title |
| `tests/pause.og.ts` | Escape holds the world (the clock and the foes); the settings from the pause; 简体中文; Done; resume; Leave the arena |
| `tests/duel.og.ts` | two seats: Duel, rounds that hit the other trooper, a frag, the respawn away from the shooter; alone: no Duel offered |
| `tests/phone.og.ts` | 1170 × 2532 (390 × 844 at 3×): the menu and HUD inside the safe area, touch-sized controls clear of the crosshair; the stick walks, a drag turns the view, Fire fires, Jump jumps, Reload reloads; the frame against `references/<tier>/<view>-phone-play.png` |
| `tests/frames.og.ts` | the title and the fight against `references/<tier>/<view>-title.png` and `-fight.png` (first person, third person, isometric and the first-person split); `OG_FX_CAPTURE=1 og test .` records them |
| `tests/replay.og.ts` | a run with movement, a jump, firing, a kill and a drone's bolt, run twice, ends on one hash |
| `tests/room.og.ts` | (online scaffolds, written by og new) two processes in one room, each seat on its own bot, hash the same at tick 600 |

## Assets

`tools/make-assets.py` regenerates the arena's scene and navmesh, the
rifle and the drone, the deck and decal textures, the sounds, music and
`og sfx` presets byte for byte; the cover is KayKit's CC0 Space Base Bits
(glTF packed to glb), the characters, the clip library and the effect
textures are the engine's own CC0 assets. `LICENSES.md` lists every file;
the style's are in `assets/styles/scifi/LICENSES.md`.

## Known limits (engine gaps, not design)

- With two seats on one screen each half builds the kit's crosshair (hit
  markers and all) in place at its own centre (`anchor: null`), but the
  kit's `damageDirection` (always centred on the frame) and
  `lowHealthVignette` (the whole frame) take no anchor, so they are drawn
  with one seat alone.
- The starter's controls have no Menu button, so the summary's Title and
  the pause's Leave the arena press Reload (it does nothing else on those
  screens).
- No CC0 rigged robot was in the catalogue, so the walkers are the
  engine's stout humanoid, tinted and visored.

# The game template — the yard (3D)

What `og new` writes for a 3D game: the hello of the engine's design grown
into a small game you can play. The title is the scene it always was: a
menu panel, the trickster beside the dropship, a crate held in the air and a
guard on its rounds (`timeline:play:intro` plays the intro over it). Start
(the button, S, Enter or a pad's A) drops the crate, spins the dropship up
as it lifts off the pad, raises a fence round the yard and hands the
trickster to the player. Walk, run and jump it through the yard, gather the
five power cells that pop up round it and stand on the lit pad under the
dropship to escape. The guard patrols its post, sees the nearest hero
inside its view cone and not through the wall, raises the alarm and runs
it down: its grab costs a third of the hero's Health and puts it back by
the pad, and a hero with none left loses the run. Then the summary, and
again.

院子：标题画面就是原来的场景——菜单、捣蛋鬼、运输船、悬在空中的箱子和巡逻的守卫。按下开始后，箱子落下，运输船旋转着升空，院子四周升起围栏，捣蛋鬼交给玩家操控：在院子里走、跑、跳，收集五个能量块，然后站上运输船下方亮起的平台逃脱。守卫会追赶视野内最近的英雄，被抓住会失去三分之一的生命并被送回平台旁；生命耗尽则失败。随后是总结，再来一次。

```sh
og run .     # headless with the probe; og dev . reloads on save
og test .    # tests/*.og.ts through og run --headless --commands
og polish .  # the taste and feel scores of those playtests
og build .   # the play page
og run . --headless --commands "0.3:key_down:KeyS;0.4:key_up:KeyS;0.6:key_down:KeyW;2:screenshot:walk.png;2.1:quit"
```

## Rules

- **Win**: every cell in (`Mission.cells` reaches 5), then a hero within
  `Tuning.padRadius` of the pad's centre: `escaped`, the time kept as the
  best when it is faster.
- **Lose**: a hero grabbed with no Health left: `caught`.
- A grab costs `Tuning.grabDamage` (34 of 100), returns the hero to its
  start and makes it ungrabbable for 2 s (it blinks); the guard goes back
  to its patrol.

## Views and seats

The view is a creation choice: `src/creation.ts` (written by `og new` from
packages/create's starters.ts, never by hand) names it, and `src/view.ts`
(the camera rig every 3D starter shares) draws it:

| View | Camera | The move | The look |
|---|---|---|---|
| `third-person-3d` (default) | `og.FollowCamera` 4.5 m behind and 1.6 m above the hero, looking at its head | relative to the view | turns the camera round the hero and tips it |
| `first-person-3d` | `og.FirstPersonCamera` at the eye, 1.6 m up; the hero's own body is on its seat's layer (`bodyLayer`), which its own eye does not draw, so its shadow still falls; a cell lantern in its right hand (`Hands`, on the seat's hands layer, drawn by its own eye alone) | relative to the view | turns the head and the body |
| `isometric-2.5d` | an orthographic `og.FollowCamera` at the look's ArtCamera angle (35.264° down, turned 45°) | relative to the screen: up walks up the screen | turns the hero only |

Every hero is a seat (`src/seats.ts`, the same file in every starter): the
game reads `og.PlayerInput` alone, so the yard plays

- **alone**: seat 0 from the keys (both sets), any pad, the mouse while the
  pointer is locked, and on a touch screen the move stick bottom left, the
  look stick with Run and Jump bottom right;
- **local-multiplayer**: a second walker (`trickster-1`, its cloth tinted
  blue) on the arrows, Right Shift and Enter (or pad 2) beside seat 0 on
  WASD, Left Shift and Space (or pad 1); the screen splits in halves, an
  `og.CameraView` per seat, each following its own hero in the chosen view;
- **online-lockstep**: two peers over a room, each peer's camera following
  its own seat (the controllers' `seat`, `og.Lockstep.localPlayer`), every
  peer running the whole yard.

At Start first and third person look north-west into the yard (`START_YAW`
in `src/play.ts`): a cell ahead, the guard's rounds between, the pad, the
crate and the wall's end on the right; the dropship hovers 6 m up, above
both views and high over the pad isometric.

It is co-op: the cells and the pad are shared, either hero can escape, and
the guard hunts the nearest hero it sees. Buttons that change the world go
through `press()`: the title's Start and the summary's Play again press the
`start` control, the HUD's Pause and the pause's Resume press `pause`, so a
room acts on one tick on every peer. `bot:<n>` drives this device's seats
from a script (the room test).

## Components, resources and systems

| Name | Where | What |
|---|---|---|
| `Health`, `Velocity`, `Spin`, `Run`, `Alarm` | `src/components.ts` | the hello's state; `Spin` is on the dropship, `Run.started` is the title's Start |
| `Mission`, `Tuning` | on the dropship | the run (phase, ending, cells, clock, grabs, record, sound bits) and its tunables |
| `Hero`, `SeatLook` | on each hero | its seat, facing, clip and press edges; its look (src/view.ts) |
| `Cell`, `Rise`, `Voice` | `src/components.ts` | a power cell; a height eased over time (the fence, the lift); a sound voice |
| `Seats` | `src/seats.ts` | every device's input into each seat's `og.PlayerInput` |
| `Flow` | `src/play.ts` | Start on the title and the summary, Pause; Start's tick builds the run (`beginRun`) |
| `HeroMove` | `src/play.ts` | the capsule's walk, run and jump on `og.CharacterController`, the body's turn, idle/walk/run clips, steps |
| `ViewRig`, then `og.systems.cameras` | `src/view.ts` | each seat's look and its camera or split-screen half |
| `Yard` | `src/play.ts` | cells taken, the guard's grab, the lit pad and the escape |
| `Hands` | `src/play.ts` | first person: each seat's lantern before its eye, swaying with its steps |
| `Voices`, `Hud` | `src/play.ts`, `src/hud.ts` | the sounds; the title, HUD, pause, settings and summary |
| `Move`, `StartSpin`, `Drop`, `Chime`, `Rotate`, `Lift` | `src/main.ts`, `src/play.ts` | the hello: the dropship spins up, the crate falls and chimes, the lifts ease |

The guard is data: a state machine in `assets/ai/guard.ogt` (og.ai), a
navmesh baked from `assets/nav/yard.ogt` (og.nav), `og.Perception`'s cone
against the wall. The intro is `assets/timelines/intro.ogt`. The settings
(volume, effects, language, reduced motion) are this device's own
(`prefs` in `src/hud.ts`), saved through og.platform, never in the world.

## Tunables

`Tuning` on the dropship (the inspector, or
`set:dropship.Tuning.<field>=<value>`): `walkSpeed` 2.6, `runSpeed` 5.2,
`jumpSpeed` 5.2 m/s, `catchReach` 1.7 m, `grabDamage` 34, `padRadius`
1.8 m; `Spin.target` 1.5 rad/s; the guard's sight and cone on its
`og.Perception`.

## Commands

`spawn_wave:[n]`, `lure:<x>:<z>` (the trickster), `warp:<seat>:<x>:<z>`,
`health:<seat>:<hp>`, `bot:<n>`, `viewport:<desktop|phone>`,
`language:<en|zh-CN>`, `alarm` (the guard's machine); `help` lists them.

## Playtests

| Test | What |
|---|---|
| `tests/smoke.og.ts` | a tap on Start and a second of W in this scaffold's view: the walk, the clip, the lift, the crate, and the framing for the view; its frame against `references/<tier>/smoke-<view>.png` |
| `tests/seats.og.ts` | keys, pads and the touch sticks reach seat 0's (alone) or each seat's (local) `og.PlayerInput` and move only that hero |
| `tests/run.og.ts` | the loop's two ends: a grab costs Health and blinks the hero, a grab with none left loses, Play again starts afresh, every cell and the pad win |
| `tests/guard.og.ts` | the guard patrols, sees inside its cone and not through the wall, chases, searches, returns |
| `tests/intro.og.ts` | the intro's rail camera, captions, clips, event and fades; `intro-4.png`, `intro-8.png` |
| `tests/inspector*.og.ts` | the inspector edits the crate and its gizmo drags it; `inspector-*.png` |

Each reads the scaffold's choices from `../src/creation.ts`
(`tests/lib.ts`), so it passes at every view and player count;
`OG_FX_CAPTURE=1 og test .` records the frames of this tier.

## What a Maker changes first

1. The premise: replace the cells with what your hero gathers (`CELL_AT`
   and `beginRun` in `src/play.ts`) and the pad with your goal.
2. The guard: its routes and states in `assets/ai/guard.ogt`, more guards
   in `start`, and the yard in `assets/nav/yard.ogt` when you move the wall.
3. The hero: a model of your own in place of `models/walker.glb` (its
   clips named idle, walk and run) and `Tuning`'s speeds.
4. The look (`assets/looks/`), the strings (`assets/strings/`) and the
   sounds (`assets/audio/*.ogt`, og sfx presets).

Asset licences: `LICENSES.md`.
